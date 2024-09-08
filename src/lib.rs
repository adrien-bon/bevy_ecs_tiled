//! [`bevy_ecs_tiled`](https://github.com/adrien-bon/bevy_ecs_tiled) is a [`Bevy`](https://bevyengine.org/) plugin for working with 2D tilemaps created with the [Tiled](https://www.mapeditor.org/) map editor.
//!
//! It uses the [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap) crate to perform rendering, so each tile is represented by a Bevy entity:
//! - layers are children of the tilemap entity
//! - tiles and objects are children of layers
//!
//! `Visibility` and `Transform` are inherited: map -> layer -> tile / object
//!
//! ## Features
//!
//! - Orthogonal, isometric and hexagonal maps
//! - Finite and infinite maps
//! - Embedded and separate tileset
//! - Easily spawn / despawn maps
//! - Animated tiles
//! - Rapier and Avian colliders added from tilesets and object layers (`rapier` or `avian` feature flag)
//! - Tiled custom properties mapped to Bevy components (`user_properties` feature flag)
//!
//! ## Getting started
//!
//! ```toml
//! [dependencies]
//! bevy = "0.14"
//! bevy_ecs_tiled = "0.3"
//! bevy_ecs_tilemap = "0.14"
//! ```
//!
//! Then add the plugin to your app:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_ecs_tiled::prelude::*;
//! use bevy_ecs_tilemap::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(TilemapPlugin)
//!         .add_plugins(TiledMapPlugin)
//!         .add_systems(Startup, startup)
//!         .run();
//! }

//! fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     // Spawn a 2D camera
//!     commands.spawn(Camera2dBundle::default());
//!
//!     // Ensure any tile / tileset paths are relative to assets/
//!     let map_handle: Handle<TiledMap> = asset_server.load("map.tmx");
//!     commands.spawn(TiledMapBundle {
//!         tiled_map: map_handle,
//!         ..Default::default()
//!     });
//! }
//! ```
//!
//! See documentation for [components::TiledMapBundle] and the [examples](https://github.com/adrien-bon/bevy_ecs_tiled/examples/README.md) for more advanced use cases.
//!

pub mod components;
pub mod debug;
pub mod loader;
pub mod names;
pub mod physics;
pub mod utils;

#[cfg(feature = "user_properties")]
pub mod properties;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::debug::*;
    pub use crate::loader::*;
    pub use crate::names::*;
    pub use crate::physics::*;
    #[cfg(feature = "user_properties")]
    pub use crate::properties::prelude::*;
    pub use crate::utils::*;
    pub use tiled;
}
