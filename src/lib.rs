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
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_code)]
#![deny(missing_debug_implementations)]
// Bevy queries with filters easily trigger this lint, without being hard to read
#![allow(clippy::type_complexity)]

pub mod asset;
pub mod ecs;
pub mod properties;
pub mod sets;

#[cfg(feature = "render")]
pub mod render;

#[cfg(feature = "debug")]
pub mod debug;

#[cfg(feature = "physics")]
pub mod physics;

mod plugin;

#[cfg(test)]
mod tests;

pub use plugin::TiledPlugin;

/// `bevy_ecs_tiled` public exports.
pub mod prelude {
    #[cfg(all(feature = "debug", feature = "physics"))]
    pub use super::debug::collider_chunks::{
        TiledDebugColliderChunksConfig, TiledDebugColliderChunksPlugin,
    };
    #[cfg(all(feature = "debug", feature = "world"))]
    pub use super::debug::world_streaming::{
        TiledDebugWorldStreamingConfig, TiledDebugWorldStreamingPlugin,
    };
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
    pub use super::render::{
        animation::TiledAnimation,
        object::{TiledObjectText, TiledObjectTextBox, TiledObjectVisualOf, TiledObjectVisuals},
        tilemap::{TiledTilemap, TiledTilemapOf, TiledTilemaps},
    };
    pub use super::{
        TiledPlugin,
        asset::{
            LayerId, ObjectId, TilesetId,
            map::{TiledMapAsset, TiledMapLoaderError},
        },
        ecs::{
            event::{
                LayerCreated, MapCreated, ObjectCreated, TileWithPropertiesCreated, TiledEvent,
                TilemapCreated,
            },
            filter::{TiledFilter, TiledName},
            image::TiledImage,
            layer::{TiledLayer, TiledLayerParallax, TiledParallaxCamera, TiledTileLayerStorage},
            map::{
                RespawnTiledMap, TiledMap, TiledMapHelper, TiledMapImageRepeatMargin,
                TiledMapLayerZOffset, TiledMapReference,
                query::{TiledMapLayers, TiledMapObjects, TiledMapQuery, TiledMapTiles},
            },
            object::TiledObject,
            tile::TiledTile,
        },
        properties::TiledProperties,
        sets::{TiledPostUpdateSystems, TiledPreUpdateSystems, TiledUpdateSystems},
    };
    #[cfg(feature = "world")]
    pub use super::{
        asset::{
            WorldMapId,
            world::{TiledWorldAsset, TiledWorldLoaderError},
        },
        ecs::{
            event::WorldCreated,
            world::{
                RespawnTiledWorld, TiledWorld,
                query::{TiledWorldMaps, TiledWorldReference},
                streaming::TiledWorldStreaming,
            },
        },
    };

    // Re-exports from `bevy_ecs_tilemap`:
    // just take everything since we are tigthly coupled
    pub use bevy_ecs_tilemap::prelude::*;
}

/// Re-export of the [`tiled`](https://crates.io/crates/tiled) crate, giving access to the raw Tiled data.
pub use tiled;

/// Re-export of the [`geo`](https://crates.io/crates/geo) crate, used to describe physics colliders geometry.
pub use geo;

/// Re-export of the [`regex`](https://crates.io/crates/regex) crate, used to build [`TiledFilter`](prelude::TiledFilter)s.
pub use regex;
