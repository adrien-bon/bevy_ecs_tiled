//! Core Tiled integration for `bevy_ecs_tiled`.
//!
//! This module contains the main logic for loading, processing, and managing Tiled maps and worlds within Bevy.
//! It organizes submodules for assets, components, systems, events and utilities related to Tiled support.

#[cfg(feature = "render")]
pub mod animation;
pub mod event;
pub mod filter;
pub mod helpers;
pub mod image;
pub mod layer;
pub mod map;
pub mod object;
pub mod properties_raw;
pub mod sets;
pub mod tile;
#[cfg(feature = "world")]
pub mod world;

pub(crate) mod cache;
pub(crate) mod reader;

#[cfg(feature = "user_properties")]
pub mod properties;

#[cfg(feature = "user_properties")]
use crate::prelude::*;
use bevy::prelude::*;
#[cfg(feature = "user_properties")]
use std::path::PathBuf;

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
///
/// With the `user_properties` feature, the plugin can export the types registered in your application
/// to a file, at startup, so you can import them in Tiled (see the
/// [dedicated book section](https://adrien-bon.github.io/bevy_ecs_tiled/guides/properties.html)):
/// ```rust,no_run
/// # #[cfg(feature = "user_properties")]
/// # {
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new().add_plugins(TiledPlugin {
///     types_export_file: Some("tiled_types_export.json".into()),
///     types_filter: TiledFilter::from(["my_game::Player", "my_game::Enemy"]),
/// });
/// # }
/// ```
#[derive(Default, Clone, Debug)]
pub struct TiledPlugin {
    /// Path of the file Tiled types are exported to, at startup.
    ///
    /// Defaults to [`None`]: types are not exported.
    #[cfg(feature = "user_properties")]
    pub types_export_file: Option<PathBuf>,
    /// Only types whose path matches this filter are exported.
    ///
    /// Defaults to [`TiledFilter::All`].
    #[cfg(feature = "user_properties")]
    pub types_filter: TiledFilter,
}

impl Plugin for TiledPlugin {
    fn build(&self, mut app: &mut App) {
        if !app.is_plugin_added::<bevy_ecs_tilemap::TilemapPlugin>() {
            app = app.add_plugins(bevy_ecs_tilemap::TilemapPlugin);
        }

        #[cfg(feature = "user_properties")]
        if let Some(path) = &self.types_export_file {
            app.insert_resource(properties::TiledTypesExport {
                path: path.clone(),
                filter: self.types_filter.clone(),
            });
        }

        app.add_plugins((
            // Must come first: asset loaders use the resource cache
            cache::plugin,
            map::plugin,
            #[cfg(feature = "world")]
            world::plugin,
            #[cfg(feature = "render")]
            animation::plugin,
            event::plugin,
            image::plugin,
            layer::plugin,
            object::plugin,
            tile::plugin,
            sets::plugin,
            filter::plugin,
            properties_raw::plugin,
            #[cfg(feature = "user_properties")]
            properties::plugin,
        ));
    }
}
