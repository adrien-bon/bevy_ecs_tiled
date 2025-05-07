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
    pub use super::TiledMapHandle;
}

use crate::{cache::TiledResourceCache, prelude::*};
use bevy::{asset::RecursiveDependencyLoadState, prelude::*};
use bevy_ecs_tilemap::prelude::*;
#[cfg(feature = "user_properties")]
use std::path::Path;

/// Wrapper around the [Handle] to the `.tmx` file representing the [TiledMap].
///
/// This is the main [Component] that must be spawned to load a Tiled map.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Debug)]
#[require(
    TiledMapStorage,
    TiledMapLayerZOffset,
    TilemapRenderSettings,
    TilemapAnchor,
    Visibility,
    Transform
)]
pub struct TiledMapHandle(pub Handle<TiledMap>);

pub(crate) fn build(app: &mut bevy::prelude::App) {
    app.init_asset::<TiledMap>()
        .init_asset_loader::<TiledMapLoader>()
        .register_type::<TiledMapHandle>()
        .register_type::<TiledMapPluginConfig>()
        .register_type::<TiledMapLayerZOffset>()
        .register_type::<RespawnTiledMap>()
        .register_type::<TiledMapStorage>()
        .register_type::<TiledMapMarker>()
        .register_type::<TiledMapLayer>()
        .register_type::<TiledMapTileLayer>()
        .register_type::<TiledMapTileLayerForTileset>()
        .register_type::<TiledMapObjectLayer>()
        .register_type::<TiledMapImageLayer>()
        .register_type::<TiledMapTile>()
        .register_type::<TiledMapObject>()
        .register_type::<TiledMapImage>()
        .register_type::<TiledAnimation>()
        .add_event::<TiledMapCreated>()
        .register_type::<TiledMapCreated>()
        .add_event::<TiledLayerCreated>()
        .register_type::<TiledLayerCreated>()
        .add_event::<TiledObjectCreated>()
        .register_type::<TiledObjectCreated>()
        .add_event::<TiledTileCreated>()
        .register_type::<TiledTileCreated>()
        .add_systems(PreUpdate, process_loaded_maps)
        .add_systems(Update, animate_tiled_sprites)
        .add_systems(PostUpdate, handle_map_events);
}

/// Export a Tiled types to the given path.
///
/// The predicate determines whether a symbol is exported. To export all
/// symbols, one can provide a blanket yes predicate, e.g. `|_| true`.
#[cfg(feature = "user_properties")]
pub fn export_types(
    reg: &AppTypeRegistry,
    path: impl AsRef<Path>,
    predicate: impl Fn(&str) -> bool,
) {
    use std::{fs::File, io::BufWriter, ops::Deref};
    let file = File::create(path).unwrap();
    let writer = BufWriter::new(file);
    let registry = crate::properties::export::TypeExportRegistry::from_registry(reg.read().deref());
    let mut list = registry.to_vec();
    list.retain(|v| predicate(&v.name));
    serde_json::to_writer_pretty(writer, &list).unwrap();
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
            &mut TiledMapStorage,
            &TilemapRenderSettings,
            &TilemapAnchor,
            &TiledMapLayerZOffset,
        ),
        Or<(
            Changed<TiledMapHandle>,
            Changed<TilemapAnchor>,
            Changed<TiledMapLayerZOffset>,
            Changed<TilemapRenderSettings>,
            With<RespawnTiledMap>,
        )>,
    >,
    mut event_writers: TiledMapEventWriters,
) {
    for (map_entity, map_handle, mut tiled_id_storage, render_settings, anchor, layer_offset) in
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
                error!("Cannot get a valid TiledMap out of Handle<TiledMap>: has the last strong reference to the asset been dropped ? (handle = {:?})", map_handle.0);
                commands.entity(map_entity).despawn();
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
                layer_offset,
                &asset_server,
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
                        commands.entity(map_entity).despawn();
                    }
                }
            }
            _ => continue,
        }
    }
}

fn remove_layers(commands: &mut Commands, tiled_id_storage: &mut TiledMapStorage) {
    for layer_entity in tiled_id_storage.layers.values() {
        commands.entity(*layer_entity).despawn();
    }
    tiled_id_storage.layers.clear();
    tiled_id_storage.objects.clear();
    tiled_id_storage.tiles.clear();
}

fn animate_tiled_sprites(
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
