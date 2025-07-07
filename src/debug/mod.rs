//! Debugging tools for bevy_ecs_tiled.
//!
//! This module provides plugins and utilities to help visualize and debug Tiled maps and worlds
//! in your Bevy application. Enable the `debug` feature to use these plugins, which include
//! gizmo overlays for objects, tiles, world chunks, and axes.

pub mod axis;
pub mod objects;
pub mod tiles;
pub mod world_chunk;

use bevy::app::{PluginGroup, PluginGroupBuilder};

/// This [`PluginGroup`] contains all debug plugins from `bevy_ecs_tiled`.
///
/// It can be used to easily turn on all debug informations :
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugPluginGroup);
/// ```
#[derive(Default, Copy, Clone, Debug)]
pub struct TiledDebugPluginGroup;

impl PluginGroup for TiledDebugPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(objects::TiledDebugObjectsPlugin::default())
            .add(tiles::TiledDebugTilesPlugin::default())
            .add(world_chunk::TiledDebugWorldChunkPlugin::default())
            .add(axis::TiledDebugAxisPlugin)
    }
}
