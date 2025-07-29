//! Physics settings management for Tiled maps and worlds.
//!
//! This module defines the [`TiledPhysicsSettings`] component, which controls how physics colliders are generated
//! for Tiled maps and worlds in Bevy. It notably provides filtering options for selecting which Tiled objects and tiles
//! should receive colliders.

use crate::prelude::*;
use bevy::prelude::*;

/// Component for configuring physics collider generation for Tiled maps and worlds.
///
/// Allows filtering which objects and tiles receive colliders.
/// Attach this component to TiledWorld or TiledMap entities to control their physics behavior.
#[derive(Component, Default, Reflect, Clone, Debug)]
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
    /// Physics backend to use for adding colliders.
    pub backend: T,
}

pub(crate) fn plugin<T: TiledPhysicsBackend>(app: &mut App) {
    app.register_type::<TiledPhysicsSettings<T>>();
    app.add_systems(
        PreUpdate,
        (
            initialize_settings_for_worlds::<T>,
            initialize_settings_for_maps::<T>,
        )
            .chain()
            .in_set(TiledPreUpdateSystems::InitializePhysicsSettings),
    );
    app.add_systems(
        PostUpdate,
        handle_settings_update::<T>.in_set(TiledPostUpdateSystems::HandlePhysicsSettingsUpdate),
    );
}

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
    worlds_query: Query<&TiledPhysicsSettings<T>, With<TiledWorld>>,
) {
    for (map, child_of) in maps_query.iter() {
        commands.entity(map).insert(
            child_of
                .and_then(|child_of| worlds_query.get(child_of.parent()).ok())
                .cloned()
                .unwrap_or_default(),
        );
    }
}

fn handle_settings_update<T: TiledPhysicsBackend>(
    mut commands: Commands,
    maps_query: Query<(Entity, Ref<TiledPhysicsSettings<T>>), With<TiledMap>>,
    worlds_query: Query<(Entity, Ref<TiledPhysicsSettings<T>>), With<TiledWorld>>,
) {
    for (world, settings) in worlds_query.iter() {
        if settings.is_changed() && !settings.is_added() {
            commands.entity(world).insert(RespawnTiledWorld);
        }
    }

    for (map, settings) in maps_query.iter() {
        if settings.is_changed() && !settings.is_added() {
            commands.entity(map).insert(RespawnTiledMap);
        }
    }
}
