//! Physics settings management for Tiled maps and worlds.
//!
//! This module defines the [`TiledPhysicsSettings`] component, which controls how physics colliders are generated
//! for Tiled maps and worlds in Bevy. It notably provides filtering options for selecting which Tiled objects and tiles
//! should receive colliders.

use super::{TiledCollidersDirty, TiledLayerColliders};
use crate::prelude::*;
use bevy::prelude::*;

/// Component for configuring physics collider generation for Tiled maps and worlds.
///
/// Allows filtering which objects and tiles receive colliders.
/// Attach this component to [`TiledWorld`] or [`TiledMap`] entities to control their physics behavior.
///
/// Modifying this component spawns again all the colliders of the map, without respawning the map itself.
/// Maps of a world inherit the settings of the world, including when they are modified.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledPhysicsSettings<T: TiledPhysicsBackend> {
    /// Specify which Tiled object to add colliders for using their layer name.
    ///
    /// Colliders will be automatically added for all objects whose containing layer name matches this filter.
    /// By default, we add colliders for all objects.
    pub objects_layer_filter: TiledFilter,
    /// Specify which Tiled object to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all objects whose name matches this filter.
    /// By default, we add colliders for all objects.
    pub objects_filter: TiledFilter,
    /// Specify which tiles collision object to add colliders for using their layer name.
    ///
    /// Colliders will be automatically added for all tiles collision objects whose layer name matches this filter.
    /// By default, we add colliders for all collision objects.
    pub tiles_layer_filter: TiledFilter,
    /// Specify which tiles collision object to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all tiles collision objects whose name matches this filter.
    /// By default, we add colliders for all collision objects.
    pub tiles_objects_filter: TiledFilter,
    /// Enables merging of colliders.
    ///
    /// When `true`, complex collision shapes (e.g., multiple overlapping or connected polygons) are merged
    /// into simpler, combined shapes using a boolean union. This improves physics performance
    /// by reducing the number of colliders and simplifying collision queries.
    ///
    /// When `false`, all collision shapes are kept as-is without simplification. This preserves exact geometry
    /// but may result in slower physics simulation for highly complex maps with many objects.
    pub simplify_geometry: bool,
    /// How colliders are created from the geometry of Tiled objects and tiles.
    ///
    /// This strategy applies to the whole map, but can be overridden for a given layer or object
    /// by inserting a [`TiledColliderStrategy`] component on its entity.
    pub collider_strategy: TiledColliderStrategy,
    /// Size, in tiles, of the chunks used to split the colliders of tiles layers.
    ///
    /// By default (ie. [`TiledColliderChunkSize::Disabled`]), all the tiles of a layer are aggregated in a single set of colliders, which
    /// is computed again whenever a tile of the layer is added, modified or removed.
    ///
    /// When a chunk size is set, each tiles layer is divided into chunks of this number of tiles and each
    /// chunk gets its own colliders, held by a [`TiledColliderChunk`] entity, child of the layer.
    /// Only the chunks whose tiles are modified are computed again, which is much faster for large maps
    /// modified at runtime. In return, colliders are not merged across chunks boundaries.
    ///
    /// This size applies to all the tiles layers of the map, but can be overridden for a given layer
    /// by inserting a [`TiledColliderChunkSize`] component on its entity.
    pub tiles_chunk_size: TiledColliderChunkSize,
    /// Physics backend to use for adding colliders.
    pub backend: T,
}

impl<T: TiledPhysicsBackend> Default for TiledPhysicsSettings<T> {
    fn default() -> Self {
        Self {
            objects_layer_filter: TiledFilter::default(),
            objects_filter: TiledFilter::default(),
            tiles_layer_filter: TiledFilter::default(),
            tiles_objects_filter: TiledFilter::default(),
            simplify_geometry: true,
            collider_strategy: TiledColliderStrategy::default(),
            tiles_chunk_size: TiledColliderChunkSize::Disabled,
            backend: T::default(),
        }
    }
}

pub(crate) fn plugin<T: TiledPhysicsBackend>(app: &mut App) {
    app.register_type::<TiledPhysicsSettings<T>>();
    app.add_systems(
        PreUpdate,
        initialize_settings_for_maps::<T>.in_set(TiledPreUpdateSystems::InitializePhysicsSettings),
    );
    app.add_systems(
        PostUpdate,
        handle_settings_update::<T>.in_set(TiledPostUpdateSystems::HandlePhysicsSettingsUpdate),
    );
    #[cfg(feature = "world")]
    app.add_systems(
        PreUpdate,
        initialize_settings_for_worlds::<T>
            .before(initialize_settings_for_maps::<T>)
            .in_set(TiledPreUpdateSystems::InitializePhysicsSettings),
    );
}

#[cfg(feature = "world")]
fn initialize_settings_for_worlds<T: TiledPhysicsBackend>(
    mut commands: Commands,
    worlds_query: Query<Entity, (With<TiledWorld>, Without<TiledPhysicsSettings<T>>)>,
) {
    for world in worlds_query.iter() {
        commands
            .entity(world)
            .insert(TiledPhysicsSettings::<T>::default());
    }
}

fn initialize_settings_for_maps<T: TiledPhysicsBackend>(
    mut commands: Commands,
    maps_query: Query<
        (Entity, Option<&ChildOf>),
        (With<TiledMap>, Without<TiledPhysicsSettings<T>>),
    >,
    #[cfg(feature = "world")] worlds_query: Query<&TiledPhysicsSettings<T>, With<TiledWorld>>,
) {
    for (map, _child_of) in maps_query.iter() {
        // A map belonging to a world inherits the world settings
        #[cfg(feature = "world")]
        let settings = _child_of
            .and_then(|child_of| worlds_query.get(child_of.parent()).ok())
            .cloned()
            .unwrap_or_default();
        #[cfg(not(feature = "world"))]
        let settings = TiledPhysicsSettings::<T>::default();
        commands.entity(map).insert(settings);
    }
}

/// Update colliders when physics settings are modified.
///
/// Colliders are derived from tiles and objects entities: there is no need to respawn the map.
fn handle_settings_update<T: TiledPhysicsBackend>(
    mut commands: Commands,
    maps_query: Query<Entity, (With<TiledMap>, Changed<TiledPhysicsSettings<T>>)>,
    mut layers_query: TiledMapLayers<&mut TiledLayerColliders<T>>,
    objects_query: TiledMapObjects<Entity>,
    #[cfg(feature = "world")] worlds_query: Query<
        (Ref<TiledPhysicsSettings<T>>, &TiledWorldStorage),
        With<TiledWorld>,
    >,
) {
    // Maps inherit the settings of their world: they are processed on next frame
    #[cfg(feature = "world")]
    for (settings, storage) in worlds_query.iter() {
        if settings.is_changed() && !settings.is_added() {
            for (_, map) in storage.maps() {
                commands
                    .entity(map)
                    .try_insert(TiledPhysicsSettings::clone(&settings));
            }
        }
    }

    for map in maps_query.iter() {
        for mut colliders in layers_query.iter_mut(map) {
            colliders.mark_all_dirty();
        }
        for object in objects_query.iter(map) {
            commands
                .entity(object)
                .try_insert(TiledCollidersDirty::<T>::default());
        }
    }
}
