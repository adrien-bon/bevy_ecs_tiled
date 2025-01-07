//! This module handles all the logic related to loading and spawning Tiled maps

pub mod asset;
pub mod components;
pub mod events;
pub mod loader;
pub mod utils;

/// `bevy_ecs_tiled` map related public exports
pub mod prelude {
    pub use super::asset::*;
    pub use super::components::*;
    pub use super::events::*;
    pub use super::utils::*;
}

use crate::{cache::TiledResourceCache, prelude::*};
use bevy::{asset::RecursiveDependencyLoadState, prelude::*};
use bevy_ecs_tilemap::prelude::*;

#[cfg(feature = "user_properties")]
pub(crate) fn export_types(reg: Res<AppTypeRegistry>, config: Res<TiledMapPluginConfig>) {
    use std::{fs::File, io::BufWriter, ops::Deref};
    if let Some(path) = &config.tiled_types_export_file {
        info!("Export Tiled types to '{:?}'", path);
        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);
        let registry =
            crate::properties::export::TypeExportRegistry::from_registry(reg.0.read().deref());
        serde_json::to_writer_pretty(writer, &registry.to_vec()).unwrap();
    }
}

/// System to spawn a map once it has been fully loaded.
#[allow(clippy::type_complexity)]
pub(crate) fn process_loaded_maps(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    maps: Res<Assets<TiledMap>>,
    mut map_query: Query<
        (
            Entity,
            &TiledMapHandle,
            &mut TiledIdStorage,
            &TilemapRenderSettings,
            &TiledMapSettings,
        ),
        Or<(Changed<TiledMapHandle>, With<RespawnTiledMap>)>,
    >,
) {
    for (map_entity, map_handle, mut tiled_id_storage, render_settings, tiled_settings) in
        map_query.iter_mut()
    {
        if let Some(load_state) = asset_server.get_recursive_dependency_load_state(&map_handle.0) {
            if !load_state.is_loaded() {
                if let RecursiveDependencyLoadState::Failed(_) = load_state {
                    error!(
                        "Map failed to load, despawn it (handle = {:?})",
                        map_handle.0
                    );
                    commands.entity(map_entity).despawn_recursive();
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
                error!("Cannot get a valid TiledMap out of Handle<TiledMap>: has the last strong reference to the asset been dropped ? (handle = {:?})", map_handle.0);
                commands.entity(map_entity).despawn_recursive();
                continue;
            };

            debug!(
                "Map has finished loading, spawn map layers (handle = {:?})",
                map_handle.0
            );

            // Clean previous map layers before trying to spawn the new ones
            remove_layers(&mut commands, &mut tiled_id_storage);
            loader::load_map(
                &mut commands,
                map_entity,
                map_handle.0.id(),
                tiled_map,
                &mut tiled_id_storage,
                render_settings,
                tiled_settings,
                &asset_server,
            );

            // Remove the respawn marker
            commands.entity(map_entity).remove::<RespawnTiledMap>();
        }
    }
}

/// System to update maps as they are changed or removed.
pub(crate) fn handle_map_events(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    map_query: Query<(Entity, &TiledMapHandle)>,
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
                        commands.entity(map_entity).despawn_recursive();
                    }
                }
            }
            _ => continue,
        }
    }
}

fn remove_layers(commands: &mut Commands, tiled_id_storage: &mut TiledIdStorage) {
    for layer_entity in tiled_id_storage.layers.values() {
        commands.entity(*layer_entity).despawn_recursive();
    }
    tiled_id_storage.layers.clear();
    tiled_id_storage.objects.clear();
    tiled_id_storage.tiles.clear();
}

pub(crate) fn animate_tiled_sprites(
    time: Res<Time>,
    mut sprite_query: Query<(&mut TiledAnimation, &mut Sprite)>,
) {
    for (mut animation, mut sprite) in sprite_query.iter_mut() {
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index += 1;
                if atlas.index >= animation.end {
                    atlas.index = animation.start;
                }
            }
        }
    }
}
