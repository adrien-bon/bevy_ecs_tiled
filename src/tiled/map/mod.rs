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

/// The component representing our Tiled map.
///
/// This is a [`Handle`] to the loaded `.tmx` file, ie. a [`TiledMapAsset`].
/// This is the main [`Component`] that must be spawned to load a Tiled map.
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

/// Specificy the Z offset between two consecutives Tiled layers.
///
/// Must be added to the [`TiledMap`] [`Entity`].
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

/// Marker [`Component`] to trigger a map respawn.
///
/// Must be added to the [`TiledMap`] [`Entity`].
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn handle_respawn(
///     mut commands: Commands,
///     map_query: Query<Entity, With<TiledMap>>,
/// ) {
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
    mut map_events: EventReader<AssetEvent<TiledMapAsset>>,
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
