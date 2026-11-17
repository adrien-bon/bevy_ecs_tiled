//! Internal helper functions and utilities for the bevy_ecs_tiled plugin.
//!
//! This module provides a collection of utility functions used throughout the crate for tasks such as
//! coordinate conversions, data extraction, and other operations related to Tiled maps and worlds.

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

/// Retrieves a [`tiled::Layer`] from a [`tiled::Map`] given a [`LayerId`], including group layers and their children.
pub(crate) fn layer_from_map(map: &tiled::Map, layer_id: LayerId) -> Option<tiled::Layer<'_>> {
    fn find_layer_by_id<'a, T: 'a + ExactSizeIterator<Item = tiled::Layer<'a>>>(
        layers: T,
        layer_id: LayerId,
    ) -> Option<tiled::Layer<'a>> {
        layers
            .filter_map(|l| {
                if l.id() == layer_id.0 {
                    return Some(l);
                }
                match l.layer_type() {
                    tiled::LayerType::Group(group_layer) => {
                        find_layer_by_id(group_layer.layers(), layer_id)
                    }
                    _ => None,
                }
            })
            .next()
    }
    find_layer_by_id(map.layers(), layer_id)
}

/// Retrieves an [`tiled::Object`] from a [`tiled::Map`] given an [`ObjectId`],
/// searching all objects layers, including the ones in group layers.
pub(crate) fn object_from_map(map: &tiled::Map, object_id: ObjectId) -> Option<tiled::Object<'_>> {
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
/// For hexagonal maps staggered along the Y axis, the [`HexCoordSystem`] also depends on
/// the parity of the map height (`tilemap_size.y`): since `bevy_ecs_tilemap` Y axis is flipped
/// compared to Tiled, Tiled row `r` becomes row `tilemap_size.y - 1 - r`, which changes
/// which rows are staggered when the map height is odd.
///
/// Returns [`None`] if the orientation is not supported by this plugin, ie.
/// [`tiled::Orientation::Staggered`] or [`tiled::Orientation::Oblique`]:
/// such maps are rejected by the map loader.
pub(crate) fn tilemap_type_from_map(
    map: &tiled::Map,
    tilemap_size: &TilemapSize,
) -> Option<TilemapType> {
    match map.orientation {
        tiled::Orientation::Orthogonal => Some(TilemapType::Square),
        tiled::Orientation::Hexagonal => {
            let odd_height = tilemap_size.y % 2 == 1;
            Some(TilemapType::Hexagon(
                match (map.stagger_axis, map.stagger_index, odd_height) {
                    (tiled::StaggerAxis::X, tiled::StaggerIndex::Even, _) => {
                        HexCoordSystem::ColumnOdd
                    }
                    (tiled::StaggerAxis::X, tiled::StaggerIndex::Odd, _) => {
                        HexCoordSystem::ColumnEven
                    }
                    (tiled::StaggerAxis::Y, tiled::StaggerIndex::Even, false)
                    | (tiled::StaggerAxis::Y, tiled::StaggerIndex::Odd, true) => {
                        HexCoordSystem::RowOdd
                    }
                    (tiled::StaggerAxis::Y, tiled::StaggerIndex::Odd, false)
                    | (tiled::StaggerAxis::Y, tiled::StaggerIndex::Even, true) => {
                        HexCoordSystem::RowEven
                    }
                },
            ))
        }
        tiled::Orientation::Isometric => Some(TilemapType::Isometric(IsoCoordSystem::Diamond)),
        tiled::Orientation::Staggered | tiled::Orientation::Oblique => None,
    }
}

/// Length of the hexagon side of a [`tiled::Map`], in pixels (`0` if not set).
pub(crate) fn hex_side_length(map: &tiled::Map) -> f32 {
    map.hex_side_length.unwrap_or(0).max(0) as f32
}

/// Computes the `bevy_ecs_tilemap` [`TilemapGridSize`] of a [`tiled::Map`].
///
/// It is the map tile size, except for hexagonal maps: `bevy_ecs_tilemap` assumes regular
/// hexagons and spaces staggered rows (or columns) by `0.75 * grid_size`, whereas Tiled spaces
/// them by `(tile_size + hexsidelength) / 2`. The grid size is adjusted along the stagger axis
/// so both match. It is unchanged when `hexsidelength` is half the tile size.
pub(crate) fn grid_size_from_map(map: &tiled::Map) -> TilemapGridSize {
    let (tile_width, tile_height) = (map.tile_width as f32, map.tile_height as f32);
    if map.orientation != tiled::Orientation::Hexagonal {
        return TilemapGridSize::new(tile_width, tile_height);
    }
    let side = hex_side_length(map);
    match map.stagger_axis {
        tiled::StaggerAxis::X => TilemapGridSize::new(2. * (tile_width + side) / 3., tile_height),
        tiled::StaggerAxis::Y => TilemapGridSize::new(tile_width, 2. * (tile_height + side) / 3.),
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
pub(crate) fn tilemap_offset(map_asset: &TiledMapAsset, tileset: &tiled::Tileset) -> Vec2 {
    let mut offset = Vec2::new(tileset.offset_x as f32, -tileset.offset_y as f32);
    if matches!(map_asset.map_type, TilemapType::Isometric(..)) {
        let grid_size = map_asset.grid_size;
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
