//! This module contains some tools to help you debug your application.
//!
//! You need to enable the `debug` feature to use it.
//!

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod objects;
pub mod tiles;

pub mod prelude {
    pub use super::TiledDebugPluginGroup;
    pub use super::objects::*;
    pub use super::tiles::*;
}

pub struct TiledDebugPluginGroup;

impl PluginGroup for TiledDebugPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(objects::TiledDebugObjectsPlugin::default())
            .add(tiles::TiledDebugTilesPlugin::default())
    }
}