//! This module contains some tools to help you debug your application.
//!
//! You need to enable the `debug` feature to use it.
//!

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod objects;
pub mod tiles;
pub mod world_chunk;

/// `bevy_ecs_tiled` debug exports.
pub mod prelude {
    pub use super::objects::*;
    pub use super::tiles::*;
    pub use super::world_chunk::*;
    pub use super::TiledDebugPluginGroup;
}

/// This [PluginGroup] contains all debug plugins from `bevy_ecs_tiled`.
///
/// It can be used to easily turn on all debug informations :
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugPluginGroup);
/// ```
pub struct TiledDebugPluginGroup;

impl PluginGroup for TiledDebugPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(objects::TiledDebugObjectsPlugin::default())
            .add(tiles::TiledDebugTilesPlugin::default())
            .add(world_chunk::TiledDebugWorldChunkPlugin::default())
    }
}
