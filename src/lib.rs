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
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

pub mod names;
pub mod tiled;

#[cfg(feature = "debug")]
pub mod debug;

#[cfg(feature = "physics")]
pub mod physics;

/// `bevy_ecs_tiled` public exports.
pub mod prelude {
    #[cfg(feature = "debug")]
    pub use super::debug::{
        axis::TiledDebugAxisPlugin,
        objects::{TiledDebugObjectsConfig, TiledDebugObjectsPlugin},
        tiles::{TiledDebugTilesConfig, TiledDebugTilesPlugin},
        world_chunk::{TiledDebugWorldChunkConfig, TiledDebugWorldChunkPlugin},
        TiledDebugPluginGroup,
    };
    pub use super::names::{TiledName, TiledNameFilter};
    #[cfg(feature = "avian")]
    pub use super::physics::backend::avian::TiledPhysicsAvianBackend;
    #[cfg(feature = "rapier")]
    pub use super::physics::backend::rapier::TiledPhysicsRapierBackend;
    #[cfg(feature = "physics")]
    pub use super::physics::{
        backend::{multi_polygon_as_line_strings, multi_polygon_as_triangles, TiledPhysicsBackend},
        collider::{ColliderCreated, TiledColliderOrigin, TiledColliderPolygons},
        settings::TiledPhysicsSettings,
        TiledPhysicsPlugin,
    };
    pub use super::tiled::{
        animation::TiledAnimation,
        event::{
            LayerCreated, MapCreated, ObjectCreated, TileCreated, TiledEvent, TilemapCreated,
            WorldCreated,
        },
        helpers::{
            get_layer_from_map, get_object_from_map, get_tile_from_map, get_tileset_from_map,
            grid_size_from_map, tile_size_from_grid_size, tile_size_from_map,
            tilemap_type_from_map,
        },
        image::TiledImage,
        layer::TiledLayer,
        map::{
            asset::TiledMapAsset, loader::TiledMapLoaderError, storage::TiledMapStorage,
            RespawnTiledMap, TiledMap, TiledMapLayerZOffset,
        },
        object::TiledObject,
        sets::{TiledPostUpdateSystems, TiledPreUpdateSystems, TiledUpdateSystems},
        tile::{TiledTile, TiledTilemap},
        world::{
            asset::TiledWorldAsset, chunking::TiledWorldChunking, loader::TiledWorldLoaderError,
            storage::TiledWorldStorage, RespawnTiledWorld, TiledWorld,
        },
        TiledPlugin, TiledPluginConfig,
    };

    // Re-exports from `bevy`
    pub use bevy::{math::bounding::Aabb2d, platform::collections::HashMap};

    // Re-exports from `bevy_ecs_tilemap`
    pub use bevy_ecs_tilemap::prelude::{
        TilePos, TilemapAnchor, TilemapGridSize, TilemapRenderSettings, TilemapSize,
        TilemapTileSize, TilemapType,
    };
    // Re-exports from `tiled`
    pub use tiled::World as TiledRawWorld; // Avoid name clash with Bevy `World`
    pub use tiled::{
        Layer, LayerTile, LayerTileData, Map, Object, Tile, TileId, TileLayer, Tileset,
    };
    // Re-exports from 'geo'
    pub use geo::Polygon as GeoPolygon; // Avoid name clash with Bevy 'Polygon'
    pub use geo::{Coord, LineString, MultiPoint, MultiPolygon, Point};
}
