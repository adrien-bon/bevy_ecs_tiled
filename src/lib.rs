#![doc = include_str!("../book/src/intro.md")]
//!
//! ## API reference
//!
//! As the name implies, this API reference purpose is to describe the API provided by `bevy_ecs_tiled`.
//!
//! For a more use-cases oriented documentation please have a look to the [`bevy_ecs_tiled` book](https://adrien-bon.github.io/bevy_ecs_tiled/) and notably the [FAQ](https://adrien-bon.github.io/bevy_ecs_tiled/FAQ.html) that will hopefully answer most of your questions.
//!
//! ## Getting started
//!
#![doc = include_str!("../book/src/getting-started.md")]

pub mod asset;
pub mod components;
pub mod debug;
pub mod events;
pub mod loader;
pub mod names;
pub mod physics;
pub mod utils;

#[cfg(feature = "user_properties")]
pub mod properties;

/// `bevy_ecs_tiled` public exports.
pub mod prelude {
    pub use super::TiledMapHandle;
    pub use super::TiledMapPlugin;
    pub use crate::asset::*;
    pub use crate::components::*;
    pub use crate::debug::*;
    pub use crate::events::*;
    pub use crate::names::*;
    #[cfg(feature = "physics")]
    pub use crate::physics::prelude::*;
    pub use crate::utils::*;
}

use crate::prelude::*;
use bevy::{asset::RecursiveDependencyLoadState, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use std::{env, path::PathBuf};

/// Wrapper around the [Handle] to the `.tmx` file representing the map.
#[derive(Component)]
pub struct TiledMapHandle(pub Handle<TiledMap>);

/// [TiledMapPlugin] [Plugin] global configuration.
#[allow(dead_code)]
#[derive(Resource, Clone)]
pub struct TiledMapPluginConfig {
    /// Path to the Tiled types export file.
    ///
    /// If [None], will not export Tiled types at startup.
    pub tiled_types_export_file: Option<PathBuf>,
}

impl Default for TiledMapPluginConfig {
    fn default() -> Self {
        let mut path = env::current_dir().unwrap();
        path.push("tiled_types_export.json");
        Self {
            tiled_types_export_file: Some(path),
        }
    }
}

/// `bevy_ecs_tiled` main `Plugin`.
///
/// This [Plugin] should be added to your application to actually be able to load a Tiled map.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledMapPlugin::default());
/// ```
#[derive(Default)]
pub struct TiledMapPlugin(pub TiledMapPluginConfig);

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TiledMap>()
            .init_asset_loader::<TiledLoader>()
            .add_systems(Update, (handle_map_events, process_loaded_maps))
            .insert_resource(self.0.clone());

        #[cfg(feature = "user_properties")]
        app.add_systems(Startup, export_types);
    }
}

#[cfg(feature = "user_properties")]
fn export_types(reg: Res<AppTypeRegistry>, config: Res<TiledMapPluginConfig>) {
    use std::{fs::File, io::BufWriter, ops::Deref};
    if let Some(path) = &config.tiled_types_export_file {
        info!("Export Tiled types to '{:?}'", path);
        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);
        let registry = properties::export::TypeExportRegistry::from_registry(reg.0.read().deref());
        serde_json::to_writer_pretty(writer, &registry.to_vec()).unwrap();
    }
}

/// System to spawn a map once it has been fully loaded.
#[allow(clippy::type_complexity)]
fn process_loaded_maps(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    maps: ResMut<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<
        (
            Entity,
            &TiledMapHandle,
            Option<&mut TiledIdStorage>,
            Option<&TilemapRenderSettings>,
            Option<&TiledMapSettings>,
        ),
        Or<(Changed<TiledMapHandle>, With<RespawnTiledMap>)>,
    >,
) {
    for (map_entity, map_handle, tiled_id_storage, render_settings, tiled_settings) in
        map_query.iter_mut()
    {
        if let Some(load_state) = asset_server.get_recursive_dependency_load_state(&map_handle.0) {
            if load_state != RecursiveDependencyLoadState::Loaded {
                // If not fully loaded yet, insert the 'Respawn' marker so we will try to load it at next frame
                commands.entity(map_entity).insert(RespawnTiledMap);
                debug!(
                    "Map '{}' is not fully loaded yet...",
                    map_handle.0.path().unwrap()
                );
                continue;
            }

            if let Some(tiled_map) = maps.get(&map_handle.0) {
                info!(
                    "Map '{}' has finished loading, spawn it",
                    map_handle.0.path().unwrap()
                );

                if let Some(mut tiled_id_storage) = tiled_id_storage {
                    debug!("Found already spawned layers, remove them");
                    remove_layers(&mut commands, &tile_storage_query, &mut tiled_id_storage);
                }
                let mut tiled_id_storage = TiledIdStorage::default();

                let render_settings = match render_settings {
                    Some(a) => a,
                    _ => {
                        commands
                            .entity(map_entity)
                            .insert(TilemapRenderSettings::default());
                        &TilemapRenderSettings::default()
                    }
                };

                let tiled_settings = match tiled_settings {
                    Some(a) => a,
                    _ => {
                        commands
                            .entity(map_entity)
                            .insert(TiledMapSettings::default());
                        &TiledMapSettings::default()
                    }
                };

                debug!("Spawn map layers");
                loader::load_map(
                    &mut commands,
                    map_entity,
                    &map_handle.0,
                    tiled_map,
                    &mut tiled_id_storage,
                    render_settings,
                    tiled_settings,
                );

                // Update ID storage and remove the respawn marker
                commands
                    .entity(map_entity)
                    .insert(tiled_id_storage)
                    .remove::<RespawnTiledMap>();
            }
        }
    }
}

/// System to update maps as they are changed or removed.
fn handle_map_events(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<(Entity, &Handle<TiledMap>, &mut TiledIdStorage)>,
    layer_query: Query<(Entity, &TiledMapLayer), With<TiledMapLayer>>,
) {
    for event in map_events.read() {
        match event {
            AssetEvent::Modified { id } => {
                log::info!("Map changed: {id}");
                for (map_entity, map_handle, _) in map_query.iter() {
                    if map_handle.id() == *id {
                        commands.entity(map_entity).insert(RespawnTiledMap);
                    }
                }
            }
            AssetEvent::Removed { id } => {
                log::info!("Map removed: {id}");
                remove_map_by_asset_id(
                    &mut commands,
                    &tile_storage_query,
                    &mut map_query,
                    &layer_query,
                    id,
                );
            }
            _ => continue,
        }
    }
}

fn remove_map_by_asset_id(
    commands: &mut Commands,
    tile_storage_query: &Query<(Entity, &TileStorage)>,
    map_query: &mut Query<(Entity, &Handle<TiledMap>, &mut TiledIdStorage)>,
    layer_query: &Query<(Entity, &TiledMapLayer), With<TiledMapLayer>>,
    asset_id: &AssetId<TiledMap>,
) {
    log::info!("removing map by asset id: {}", asset_id);
    for (_, map_handle, mut tiled_id_storage) in map_query.iter_mut() {
        log::info!("checking layer to remove: {}", map_handle.id());

        // Only process the map that was removed.
        if map_handle.id() != *asset_id {
            continue;
        }

        remove_layers(commands, tile_storage_query, &mut tiled_id_storage);
    }

    // Also manually despawn layers for this map.
    // This is necessary because when a new layer is added, the map handle
    // generation is incremented, and then a subsequent removal event will not
    // match the map_handle in the loop above.
    for (layer_entity, map_layer) in layer_query.iter() {
        // only deal with currently changed map
        if map_layer.map_handle_id != *asset_id {
            continue;
        }

        commands.entity(layer_entity).despawn_recursive();
    }
}

fn remove_layers(
    commands: &mut Commands,
    tile_storage_query: &Query<(Entity, &TileStorage)>,
    tiled_id_storage: &mut TiledIdStorage,
) {
    for layer_entity in tiled_id_storage.layers.values() {
        if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
            for tile in layer_tile_storage.iter().flatten() {
                commands.entity(*tile).despawn_recursive()
            }
        }
        commands.entity(*layer_entity).despawn_recursive();
    }
    tiled_id_storage.layers.clear();
    tiled_id_storage.objects.clear();
    tiled_id_storage.tiles.clear();
}
