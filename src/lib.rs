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

pub mod cache;
pub mod map;
pub mod names;
pub mod reader;
pub mod world;

#[cfg(feature = "debug")]
pub mod debug;

#[cfg(feature = "physics")]
pub mod physics;

#[cfg(feature = "user_properties")]
pub mod properties;

/// `bevy_ecs_tiled` public exports.
pub mod prelude {
    pub use crate::TiledMapHandle;
    pub use crate::TiledWorldHandle;
    pub use crate::TiledMapPlugin;
    pub use crate::TiledMapPluginConfig;
    pub use super::map::prelude::*;
    pub use super::names::*;
    pub use super::world::prelude::*;
    #[cfg(feature = "debug")]
    pub use super::debug::*;
    #[cfg(feature = "physics")]
    pub use super::physics::prelude::*;
}

use crate::prelude::*;
use bevy::prelude::*;
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

/// Wrapper around the [Handle] to the `.world` file representing the [TiledWorld].
///
/// This is the main [Component] that must be spawned to load a Tiled world.
#[derive(Component)]
#[require(
    TiledWorldStorage,
    TiledWorldSettings,
    TiledMapSettings,
    TilemapRenderSettings,
    Visibility,
    Transform
)]
pub struct TiledWorldHandle(pub Handle<TiledWorld>);

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
            .init_asset::<TiledWorld>()
            .insert_resource(cache::TiledResourceCache::new())
            .init_asset_loader::<TiledMapLoader>()
            .init_asset_loader::<TiledWorldLoader>()
            .add_systems(Update, (map::handle_map_events, map::process_loaded_maps, world::handle_world_events, world::process_loaded_worlds, world::world_chunking))
            .insert_resource(self.0.clone());

        #[cfg(feature = "user_properties")]
        app.add_systems(Startup, map::export_types);
    }
}
