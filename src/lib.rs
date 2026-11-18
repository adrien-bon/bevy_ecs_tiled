#![doc = include_str!("../book/src/intro.md")]
//!
//! ## Documentation
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
#![deny(missing_debug_implementations)]
#![allow(clippy::type_complexity)]

pub mod tiled;

#[cfg(feature = "debug")]
pub mod debug;

#[cfg(feature = "physics")]
pub mod physics;

#[cfg(test)]
mod tests;

/// `bevy_ecs_tiled` public exports.
pub mod prelude {
    #[cfg(all(feature = "debug", feature = "physics"))]
    pub use super::debug::collider_chunks::{
        TiledDebugColliderChunksConfig, TiledDebugColliderChunksPlugin,
    };
    #[cfg(all(feature = "debug", feature = "world"))]
    pub use super::debug::world_chunk::{TiledDebugWorldChunkConfig, TiledDebugWorldChunkPlugin};
    #[cfg(feature = "debug")]
    pub use super::debug::{
        TiledDebugGizmos, TiledDebugPluginGroup, TiledDebugTogglePlugin,
        axis::TiledDebugAxisPlugin,
        map_aabb::{TiledDebugMapAabbConfig, TiledDebugMapAabbPlugin},
        objects::{TiledDebugObjectsConfig, TiledDebugObjectsPlugin},
        tiles::{TiledDebugTilesConfig, TiledDebugTilesPlugin},
    };
    #[cfg(feature = "avian")]
    pub use super::physics::backend::avian::TiledPhysicsAvianBackend;
    #[cfg(feature = "rapier")]
    pub use super::physics::backend::rapier::TiledPhysicsRapierBackend;
    #[cfg(feature = "physics")]
    pub use super::physics::{
        TiledPhysicsDisableGeometrySimplification, TiledPhysicsPlugin,
        backend::TiledPhysicsBackend,
        collider::{
            ColliderCreated, TiledColliderChunk, TiledColliderChunkSize, TiledColliderOf,
            TiledColliderPolygons, TiledColliderSource, TiledColliderStrategy, TiledColliders,
        },
        settings::TiledPhysicsSettings,
    };
    #[cfg(feature = "render")]
    pub use super::tiled::animation::TiledAnimation;
    pub use super::tiled::{
        TiledPlugin, TiledPluginConfig,
        event::{LayerCreated, MapCreated, ObjectCreated, TileCreated, TiledEvent, TilemapCreated},
        filter::{TiledFilter, TiledName},
        helpers::tile_size,
        image::TiledImage,
        layer::{
            LayerId, TiledLayer, TiledLayerParallax, TiledParallaxCamera, TiledTileLayerStorage,
        },
        map::{
            RespawnTiledMap, TiledMap, TiledMapImageRepeatMargin, TiledMapLayerZOffset,
            TiledMapReference, asset::TiledMapAsset, loader::TiledMapLoaderError,
            storage::TiledMapStorage,
        },
        object::{
            ObjectId, TiledObject, TiledObjectText, TiledObjectTextBox, TiledObjectVisualOf,
            TiledObjectVisuals,
        },
        properties_raw::TiledProperties,
        sets::{TiledPostUpdateSystems, TiledPreUpdateSystems, TiledUpdateSystems},
        tile::{TiledTile, TiledTilemap, TiledTilemapOf, TiledTilemaps, TilesetId},
    };
    #[cfg(feature = "world")]
    pub use super::tiled::{
        event::WorldCreated,
        world::{
            RespawnTiledWorld, TiledWorld,
            asset::TiledWorldAsset,
            chunking::TiledWorldChunking,
            loader::TiledWorldLoaderError,
            storage::{TiledWorldStorage, WorldMapId},
        },
    };

    // Re-exports from `bevy_ecs_tilemap`:
    // just take everything since we are tigthly coupled
    pub use bevy_ecs_tilemap::prelude::*;

    /// Re-exports from [`tiled`](https://crates.io/crates/tiled)
    pub mod tiled {
        pub use tiled::*;
    }
}

/// Re-export of the [`geo`](https://crates.io/crates/geo) crate, used to describe physics colliders geometry.
pub use geo;

/// Re-export of the [`regex`](https://crates.io/crates/regex) crate, used to build [`TiledFilter`](prelude::TiledFilter)s.
pub use regex;
