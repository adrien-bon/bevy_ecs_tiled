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
pub mod events;
pub mod loader;
pub mod names;
pub mod utils;

#[cfg(feature = "debug")]
pub mod debug;

#[cfg(feature = "physics")]
pub mod physics;

#[cfg(feature = "user_properties")]
pub mod properties;

/// `bevy_ecs_tiled` public exports.
pub mod prelude {
    pub use super::TiledMapHandle;
    pub use super::TiledMapPlugin;
    pub use crate::asset::*;
    pub use crate::components::*;
    #[cfg(feature = "debug")]
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

/// Wrapper around the [Handle] to the `.tmx` file representing the [TiledMap].
///
/// This is the main [Component] that must be spawned to load a Tiled map.
#[derive(Component)]
#[require(
    TiledIdStorage,
    TiledMapSettings,
    TilemapRenderSettings,
    Visibility,
    Transform
)]
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
                    error!("Map '{}' failed to load", map_handle.0.path().unwrap());
                    commands.entity(map_entity).despawn_recursive();
                    return;
                }
                // If not fully loaded yet, insert the 'Respawn' marker so we will try to load it at next frame
                commands.entity(map_entity).insert(RespawnTiledMap);
                if let Some(path) = map_handle.0.path() {
                    debug!("Map '{}' is not fully loaded yet...", path);
                } else {
                    debug!(
                        "Map with handle '{}' is not fully loaded yet...",
                        map_handle.0.id()
                    );
                }
                continue;
            }

            let tiled_map = maps.get(&map_handle.0).unwrap();
            if let Some(path) = map_handle.0.path() {
                info!("Map '{}' has finished loading, spawn it", path);
            } else {
                info!(
                    "Map with handle '{}' has finished loading, spawn it",
                    map_handle.0.id()
                );
            }

            // Clean map layers
            remove_layers(&mut commands, &mut tiled_id_storage);

            debug!("Spawn map layers");
            loader::load_map(
                &mut commands,
                map_entity,
                &map_handle.0,
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
fn handle_map_events(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    mut map_query: Query<(Entity, &TiledMapHandle)>,
) {
    for event in map_events.read() {
        match event {
            AssetEvent::Modified { id } => {
                info!("Map changed: {id}");
                for (map_entity, map_handle) in map_query.iter() {
                    if map_handle.0.id() == *id {
                        commands.entity(map_entity).insert(RespawnTiledMap);
                    }
                }
            }
            AssetEvent::Removed { id } => {
                info!("Map removed: {id}");
                for (map_entity, map_handle) in map_query.iter_mut() {
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
