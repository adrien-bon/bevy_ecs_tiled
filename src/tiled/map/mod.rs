//! Core logic and types for Tiled map support.
//!
//! This module provides the main structures, systems, and utilities for handling Tiled maps within the plugin.
//! It includes submodules for map storage, entity spawning, and other map-related functionality.
//! The types and functions defined here enable loading, managing, and interacting with Tiled maps
//! in a Bevy application.

pub mod asset;
pub mod loader;
pub(crate) mod spawn;
pub mod storage;

use crate::{
    prelude::*,
    tiled::{cache::TiledResourceCache, event::TiledEventWriters},
};
use bevy::{asset::RecursiveDependencyLoadState, prelude::*};

/// Main component for loading and managing a Tiled map in the ECS world.
///
/// Attach this component to an entity to load a Tiled map from a `.tmx` file. The inner value is a [`Handle<TiledMapAsset>`],
/// which references the loaded [`TiledMapAsset`]. This entity acts as the root for all layers, tiles, and objects spawned from the map.
///
/// Required components (automatically added with default value if missing):
/// - [`TiledMapLayerZOffset`]: Controls Z stacking order between layers.
/// - [`TiledMapImageRepeatMargin`]: Controls image tiling margin for repeated images.
/// - [`TilemapRenderSettings`]: Controls custom parameters for the render pipeline.
/// - [`TilemapAnchor`]: Controls the anchor point of the map.
/// - [`Visibility`] and [`Transform`]: Standard Bevy components.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
///     commands.spawn(TiledMap(asset_server.load("map.tmx")));
/// }
/// ```
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Debug)]
#[require(
    TiledMapStorage,
    TiledMapLayerZOffset,
    TiledMapImageRepeatMargin,
    TilemapRenderSettings,
    TilemapAnchor,
    Visibility,
    Transform
)]
pub struct TiledMap(pub Handle<TiledMapAsset>);

/// Controls the Z stacking order between Tiled map layers for correct rendering.
///
/// Attach this component to a [`TiledMap`] entity to specify the Z offset (distance) between each Tiled layer.
/// This ensures that layers are rendered in the correct order and helps prevent Z-fighting artifacts, especially
/// in isometric or multi-layered maps. The value is in world units (typically pixels for 2D maps).
///
/// Defaults to `100.0` units between layers, which is suitable for most 2D maps.
/// - Increase the value if you observe rendering artifacts or Z-fighting between layers.
/// - Decrease the value if you want a more compact stacking of layers (e.g., for a "flatter" map).
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
///     commands.spawn((
///         TiledMap(asset_server.load("map.tmx")),
///         TiledMapLayerZOffset(200.0), // Custom Z offset between layers
///     ));
/// }
/// ```
///
/// # Notes
/// - The Z offset is applied incrementally for each layer: the first layer is at Z=0, the next at Z=offset, etc.
#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledMapLayerZOffset(pub f32);

impl Default for TiledMapLayerZOffset {
    fn default() -> Self {
        Self(100.)
    }
}

/// Number of extra tiles to repeat beyond the visible area on each side when tiling an
/// image with repeat x and/or repeat y enabled.
///
/// This margin ensures that, even if the camera moves or the visible area changes slightly,
/// there will be no visible gaps at the edges of the repeated image. The value represents
/// how many additional tiles (in both directions) are rendered outside the current viewport.
/// Increase this value if you observe gaps when moving the camera quickly or zooming out.
#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledMapImageRepeatMargin(pub u32);

impl Default for TiledMapImageRepeatMargin {
    fn default() -> Self {
        Self(1)
    }
}

/// Component that stores a reference to the parent Tiled map entity for a given Tiled item.
///
/// This component is automatically attached to all entities that are part of a Tiled map hierarchy,
/// such as layers [`TiledLayer`], tilemaps [`TiledTilemap`], objects [`TiledObject`], and images [`TiledImage`].
/// It allows systems and queries to easily retrieve the root map entity associated with any Tiled sub-entity.
#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Debug)]
pub struct TiledMapReference(pub Entity);

/// Marker component to trigger a respawn (reload) of a Tiled map.
///
/// Add this component to the entity holding the [`TiledMap`] to force the map and all its layers, tiles, and objects to be reloaded.
/// This is useful for hot-reloading, resetting, or programmatically refreshing the map state at runtime.
///
/// When present, the plugin will despawn all child entities and re-instantiate the map from its asset, preserving the top-level entity and its components.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn respawn_map(mut commands: Commands, map_query: Query<Entity, With<TiledMap>>) {
///     if let Ok(entity) = map_query.single() {
///         commands.entity(entity).insert(RespawnTiledMap);
///     }
/// }
/// ```
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct RespawnTiledMap;

pub(crate) fn plugin(app: &mut bevy::prelude::App) {
    app.register_type::<TiledMap>();
    app.register_type::<TiledMapLayerZOffset>();
    app.register_type::<TiledMapImageRepeatMargin>();
    app.register_type::<TiledMapReference>();
    app.register_type::<RespawnTiledMap>();

    app.add_systems(
        PreUpdate,
        process_loaded_maps.in_set(TiledPreUpdateSystems::ProcessLoadedMaps),
    );
    app.add_systems(
        PostUpdate,
        handle_map_events.in_set(TiledPostUpdateSystems::HandleMapAssetEvents),
    );

    app.add_plugins((asset::plugin, loader::plugin, storage::plugin));
}

/// System to spawn a map once it has been fully loaded.
fn process_loaded_maps(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    maps: Res<Assets<TiledMapAsset>>,
    mut map_query: Query<
        (
            Entity,
            &TiledMap,
            &mut TiledMapStorage,
            &TilemapRenderSettings,
            &TilemapAnchor,
            &TiledMapLayerZOffset,
        ),
        Or<(
            Changed<TiledMap>,
            Changed<TilemapAnchor>,
            Changed<TiledMapLayerZOffset>,
            Changed<TilemapRenderSettings>,
            With<RespawnTiledMap>,
        )>,
    >,
    mut event_writers: TiledEventWriters,
) {
    for (map_entity, map_handle, mut tiled_storage, render_settings, anchor, layer_offset) in
        map_query.iter_mut()
    {
        if let Some(load_state) = asset_server.get_recursive_dependency_load_state(&map_handle.0) {
            if !load_state.is_loaded() {
                if let RecursiveDependencyLoadState::Failed(_) = load_state {
                    error!(
                        "Map failed to load, despawn it (handle = {:?})",
                        map_handle.0
                    );
                    commands.entity(map_entity).despawn();
                } else {
                    debug!(
                        "Map is not fully loaded yet, will try again next frame (handle = {:?})",
                        map_handle.0
                    );
                    commands.entity(map_entity).insert(RespawnTiledMap);
                }
                continue;
            }

            // Map should be loaded at this point
            let Some(tiled_map) = maps.get(&map_handle.0) else {
                error!("Cannot get a valid TiledMapAsset out of Asset<TiledMapAsset>: has the last strong reference to the asset been dropped ? (handle = {:?})", map_handle.0);
                commands.entity(map_entity).despawn();
                continue;
            };

            debug!(
                "Map has finished loading, spawn map layers (handle = {:?})",
                map_handle.0
            );

            // Clean previous map layers before trying to spawn the new ones
            tiled_storage.clear(&mut commands);
            spawn::spawn_map(
                &mut commands,
                map_entity,
                map_handle.0.id(),
                tiled_map,
                &mut tiled_storage,
                render_settings,
                layer_offset,
                &mut event_writers,
                anchor,
            );

            // Remove the respawn marker
            commands.entity(map_entity).remove::<RespawnTiledMap>();
        }
    }
}

/// System to update maps as they are changed or removed.
fn handle_map_events(
    mut commands: Commands,
    mut map_events: MessageReader<AssetEvent<TiledMapAsset>>,
    map_query: Query<(Entity, &TiledMap)>,
    mut cache: ResMut<TiledResourceCache>,
) {
    for event in map_events.read() {
        match event {
            AssetEvent::Modified { id } => {
                info!("Map changed: {id}");
                // Note: this call actually clear the cache for the next time we reload an asset
                // That's because the AssetEvent::Modified is sent AFTER the asset is reloaded from disk
                // It means that is the first reload is triggered by a tileset modification, the tileset will
                // not be properly updated since we will still use its previous version in the cache
                cache.clear();
                for (map_entity, map_handle) in map_query.iter() {
                    if map_handle.0.id() == *id {
                        commands.entity(map_entity).insert(RespawnTiledMap);
                    }
                }
            }
            AssetEvent::Removed { id } => {
                info!("Map removed: {id}");
                for (map_entity, map_handle) in map_query.iter() {
                    if map_handle.0.id() == *id {
                        commands.entity(map_entity).despawn();
                    }
                }
            }
            _ => continue,
        }
    }
}
