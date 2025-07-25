//! Core Tiled integration for `bevy_ecs_tiled`.
//!
//! This module contains the main logic for loading, processing, and managing Tiled maps and worlds within Bevy.
//! It organizes submodules for assets, components, systems, events and utilities related to Tiled support.

pub mod animation;
pub(crate) mod cache;
pub mod event;
pub mod filter;
pub mod helpers;
pub mod image;
pub mod layer;
pub mod map;
pub mod object;
pub(crate) mod reader;
pub mod sets;
pub mod tile;
pub mod world;

#[cfg(feature = "user_properties")]
pub mod properties;

use crate::prelude::*;
use bevy::prelude::*;
use std::{env, path::PathBuf};

/// [`TiledPlugin`] global configuration.
///
/// Example:
/// ```rust,no_run
/// use std::env;
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// let mut path = env::current_dir().unwrap();
/// path.push("my_tiled_export_file.json");
///
/// App::new()
///     .add_plugins(TiledPlugin(TiledPluginConfig {
///         tiled_types_export_file: Some(path),
///         tiled_types_filter: TiledFilter::All,
///     }));
/// ```
#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledPluginConfig {
    /// Path to the Tiled types export file.
    ///
    /// If [`None`], will not export Tiled types at startup.
    pub tiled_types_export_file: Option<PathBuf>,
    /// Tiled types filter
    ///
    /// Only types matching this filter will be exported at startup.
    pub tiled_types_filter: TiledFilter,
}

impl Default for TiledPluginConfig {
    fn default() -> Self {
        let mut path = env::current_dir().unwrap();
        path.push("tiled_types_export.json");
        Self {
            tiled_types_export_file: Some(path),
            tiled_types_filter: TiledFilter::All,
        }
    }
}

/// `bevy_ecs_tiled` main [`Plugin`].
///
/// This [`Plugin`] should be added to your application to actually be able to load a Tiled map.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledPlugin::default());
/// ```
#[derive(Default, Clone, Debug)]
pub struct TiledPlugin(pub TiledPluginConfig);

impl Plugin for TiledPlugin {
    fn build(&self, mut app: &mut App) {
        if !app.is_plugin_added::<bevy_ecs_tilemap::TilemapPlugin>() {
            app = app.add_plugins(bevy_ecs_tilemap::TilemapPlugin);
        }

        app.insert_resource(self.0.clone());
        app.insert_resource(cache::TiledResourceCache::new());
        app.register_type::<TiledPluginConfig>();

        app.add_plugins((
            map::plugin,
            world::plugin,
            animation::plugin,
            cache::plugin,
            event::plugin,
            image::plugin,
            layer::plugin,
            object::plugin,
            tile::plugin,
            sets::plugin,
            filter::plugin,
            #[cfg(feature = "user_properties")]
            properties::plugin,
        ));
    }
}
