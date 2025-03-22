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
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_code)]
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

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
    #[cfg(feature = "debug")]
    pub use super::debug::prelude::*;
    pub use super::map::prelude::*;
    pub use super::names::*;
    #[cfg(feature = "physics")]
    pub use super::physics::prelude::*;
    pub use super::world::prelude::*;
    pub use crate::TiledMapPlugin;
    pub use crate::TiledMapPluginConfig;
}

use crate::prelude::*;
use bevy::prelude::*;
use std::{env, path::PathBuf};

#[cfg(feature = "user_properties")]
use std::path::Path;
/// [TiledMapPlugin] [Plugin] global configuration.
#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Debug)]
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
#[derive(Default, Clone, Debug)]
pub struct TiledMapPlugin(pub TiledMapPluginConfig);

impl Plugin for TiledMapPlugin {
    fn build(&self, mut app: &mut App) {
        if !app.is_plugin_added::<bevy_ecs_tilemap::TilemapPlugin>() {
            app = app.add_plugins(bevy_ecs_tilemap::TilemapPlugin);
        }
        app.insert_resource(cache::TiledResourceCache::new())
            .insert_resource(self.0.clone())
            .register_type::<TiledMapPluginConfig>()
            .add_plugins((map::build, world::build));
        #[cfg(feature = "user_properties")]
        app.add_systems(Startup, |reg: Res<AppTypeRegistry>, config: Res<TiledMapPluginConfig>| {
            if let Some(path) = &config.tiled_types_export_file {
                map::export_types(&reg, path);
            }
        });
    }
}
