//! Internal helper functions and utilities for the bevy_ecs_tiled plugin.
//!
//! This module provides a collection of utility functions used throughout the crate for tasks such as
//! coordinate conversions, data extraction, and other operations related to Tiled maps and worlds.

use std::sync::Arc;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::{HexCoordSystem, IsoCoordSystem};

/// Implements conversions from and to [`u32`] and [`std::fmt::Display`] for an ID newtype.
macro_rules! impl_id {
    ($id:ident) => {
        impl From<u32> for $id {
            fn from(id: u32) -> Self {
                Self(id)
            }
        }

        impl From<$id> for u32 {
            fn from(id: $id) -> Self {
                id.0
            }
        }

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}
pub(crate) use impl_id;

/// Retrieves a [`tiled::Layer`] from a [`tiled::Map`] given a [`LayerId`].
///
/// Returns `Some(tiled::Layer)` if the layer exists, or `None` if it does not
pub fn layer_from_map(map: &tiled::Map, layer_id: LayerId) -> Option<tiled::Layer<'_>> {
    fn find_layer_by_id<'a, T: 'a + ExactSizeIterator<Item = tiled::Layer<'a>>>(
        layers: T,
        layer_id: LayerId,
    ) -> Option<tiled::Layer<'a>> {
        layers
            .filter_map(|l| match l.layer_type() {
                tiled::LayerType::Group(group_layer) => {
                    find_layer_by_id(group_layer.layers(), layer_id)
                }
                _ => {
                    if l.id() == layer_id.0 {
                        Some(l)
                    } else {
                        None
                    }
                }
            })
            .next()
    }
    find_layer_by_id(map.layers(), layer_id)
}

/// Retrieves a [`tiled::Tileset`] from a [`tiled::Map`] given a [`TilesetId`].
///
/// Returns a reference to the tileset if found, or `None` if the ID is invalid.
pub fn tileset_from_map(map: &tiled::Map, tileset_id: TilesetId) -> Option<&Arc<tiled::Tileset>> {
    for (id, tileset) in map.tilesets().iter().enumerate() {
        if id == tileset_id.0 as usize {
            return Some(tileset);
        }
    }
    None
}

/// Retrieves a [`tiled::Tile`] from a [`tiled::Map`] given a [`TilesetId`] and a [`tiled::TileId`].
///
/// Returns `Some(tiled::Tile)` if the tile exists in the specified tileset, or `None` otherwise.
pub fn tile_from_map(
    map: &tiled::Map,
    tileset_id: TilesetId,
    tile_id: tiled::TileId,
) -> Option<tiled::Tile<'_>> {
    tileset_from_map(map, tileset_id).and_then(|t| t.get_tile(tile_id))
}

/// Retrieves an [`tiled::Object`] from a [`tiled::Map`] given an [`ObjectId`].
///
/// Searches all object layers for the specified object ID and returns it if found.
pub fn object_from_map(map: &tiled::Map, object_id: ObjectId) -> Option<tiled::Object<'_>> {
    fn find_object_by_id<'a, T: 'a + ExactSizeIterator<Item = tiled::Layer<'a>>>(
        layers: T,
        object_id: ObjectId,
    ) -> Option<tiled::Object<'a>> {
        layers
            .filter_map(|l| match l.layer_type() {
                tiled::LayerType::Group(group_layer) => {
                    find_object_by_id(group_layer.layers(), object_id)
                }
                tiled::LayerType::Objects(object_layer) => {
                    object_layer.objects().find(|o| o.id() == object_id.0)
                }
                _ => None,
            })
            .next()
    }
    find_object_by_id(map.layers(), object_id)
}

/// Converts a [`tiled::Map`]'s [`tiled::Orientation`] to a [`TilemapType`].
///
/// Returns [`None`] if the orientation is not supported by this plugin, ie.
/// [`tiled::Orientation::Staggered`] or [`tiled::Orientation::Oblique`].
pub fn try_tilemap_type_from_map(map: &tiled::Map) -> Option<TilemapType> {
    match map.orientation {
        tiled::Orientation::Orthogonal => Some(TilemapType::Square),
        tiled::Orientation::Hexagonal => Some(TilemapType::Hexagon(
            match (map.stagger_axis, map.stagger_index) {
                (tiled::StaggerAxis::X, tiled::StaggerIndex::Even) => HexCoordSystem::ColumnOdd,
                (tiled::StaggerAxis::X, tiled::StaggerIndex::Odd) => HexCoordSystem::ColumnEven,
                (tiled::StaggerAxis::Y, tiled::StaggerIndex::Even) => HexCoordSystem::RowOdd,
                (tiled::StaggerAxis::Y, tiled::StaggerIndex::Odd) => HexCoordSystem::RowEven,
            },
        )),
        tiled::Orientation::Isometric => Some(TilemapType::Isometric(IsoCoordSystem::Diamond)),
        tiled::Orientation::Staggered | tiled::Orientation::Oblique => None,
    }
}

/// Converts a [`tiled::Map`]'s [`tiled::Orientation`] to a [`TilemapType`].
///
/// Maps with an unsupported orientation are rejected by the map loader with
/// [`TiledMapLoaderError::UnsupportedOrientation`], so this function never panics
/// for a map coming from a [`TiledMapAsset`].
///
/// # Panics
/// Panics if the orientation is [`tiled::Orientation::Staggered`] or [`tiled::Orientation::Oblique`],
/// which are not supported by this plugin. See [`try_tilemap_type_from_map`] for a non-panicking version.
pub fn tilemap_type_from_map(map: &tiled::Map) -> TilemapType {
    try_tilemap_type_from_map(map)
        .unwrap_or_else(|| panic!("{:?} map orientation is not supported", map.orientation))
}

/// Converts a [`tiled::Map`]'s grid size to a [`TilemapGridSize`].
pub fn grid_size_from_map(map: &tiled::Map) -> TilemapGridSize {
    TilemapGridSize {
        x: map.tile_width as f32,
        y: map.tile_height as f32,
    }
}

/// Get the [`TilemapTileSize`] from given [`tiled::Tile`]
pub fn tile_size(tile: &tiled::Tile) -> TilemapTileSize {
    match &tile.image {
        // tile is in image collection
        Some(image) => TilemapTileSize::new(image.width as f32, image.height as f32),
        // tile is in atlas image
        None => TilemapTileSize::new(
            tile.tileset().tile_width as f32,
            tile.tileset().tile_height as f32,
        ),
    }
}

/// Offset to apply to a tilemap so its tiles are rendered at the same place than in Tiled.
///
/// This offset includes the tileset `tileoffset` and, for isometric maps, compensates for
/// `bevy_ecs_tilemap` centering tiles images on their grid cell whereas Tiled aligns them
/// on the cell bottom-left corner.
#[allow(dead_code)] // Not used when no feature is enabled
pub(crate) fn tilemap_offset(map: &tiled::Map, tileset: &tiled::Tileset) -> Vec2 {
    let mut offset = Vec2::new(tileset.offset_x as f32, -tileset.offset_y as f32);
    if matches!(tilemap_type_from_map(map), TilemapType::Isometric(..)) {
        let grid_size = grid_size_from_map(map);
        offset += Vec2::new(
            tileset.tile_width as f32 - grid_size.x,
            tileset.tile_height as f32 - grid_size.y,
        ) / 2.;
    }
    offset
}

/// Projects Tiled isometric coordinates into scalar coordinates for Bevy.
///
/// Used to convert isometric tile coordinates into world-space positions for rendering.
///
/// # Arguments
/// - `coords`: The isometric coordinates to project.
/// - `tilemap_size`: The size of the tilemap.
/// - `grid_size`: The size of each tile on the grid in pixels.
///
/// # Returns
/// The projected 2D coordinates as a [`Vec2`].
pub(crate) fn iso_projection(
    coords: Vec2,
    tilemap_size: &TilemapSize,
    grid_size: &TilemapGridSize,
) -> Vec2 {
    let fract = Vec2 {
        x: coords.x / grid_size.y,
        y: coords.y / grid_size.y,
    };
    let origin_x = tilemap_size.y as f32 * grid_size.x / 2.;
    Vec2 {
        x: (fract.x - fract.y) * grid_size.x / 2. + origin_x,
        y: (fract.x + fract.y) * grid_size.y / 2.,
    }
}
