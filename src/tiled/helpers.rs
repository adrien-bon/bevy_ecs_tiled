//! Internal helper functions and utilities for the bevy_ecs_tiled plugin.
//!
//! This module provides a collection of utility functions used throughout the crate for tasks such as
//! coordinate conversions, data extraction, and other operations related to Tiled maps and worlds.

use std::sync::Arc;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::{HexCoordSystem, IsoCoordSystem};

/// Retrieves a [`Layer`] from a [`Map`] given a layer ID.
///
/// Returns `Some(Layer)` if the layer exists, or `None` if the ID is out of bounds.
pub fn get_layer_from_map(map: &tiled::Map, layer_id: u32) -> Option<tiled::Layer<'_>> {
    map.get_layer(layer_id as usize)
}

/// Retrieves a [`Tileset`] from a [`Map`] given a tileset ID.
///
/// Returns a reference to the tileset if found, or `None` if the ID is invalid.
pub fn get_tileset_from_map(map: &tiled::Map, tileset_id: u32) -> Option<&Arc<tiled::Tileset>> {
    for (id, tileset) in map.tilesets().iter().enumerate() {
        if id == tileset_id as usize {
            return Some(tileset);
        }
    }
    None
}

/// Retrieves a [`Tile`] from a [`Map`] given a tileset ID and a [`TileId`].
///
/// Returns `Some(Tile)` if the tile exists in the specified tileset, or `None` otherwise.
pub fn get_tile_from_map(
    map: &tiled::Map,
    tileset_id: u32,
    tile_id: tiled::TileId,
) -> Option<tiled::Tile<'_>> {
    get_tileset_from_map(map, tileset_id).and_then(|t| t.get_tile(tile_id))
}

/// Retrieves an [`Object`] from a [`Map`] given an object ID.
///
/// Searches all object layers for the specified object ID and returns it if found.
pub fn get_object_from_map(map: &tiled::Map, object_id: u32) -> Option<tiled::Object<'_>> {
    for layer in map.layers() {
        let obj = layer
            .as_object_layer()
            .and_then(|l| l.objects().find(|o| o.id() == object_id));
        if obj.is_some() {
            return obj;
        }
    }
    None
}

/// Converts a [`Map`]'s [`Orientation`] to a [`TilemapType`].
///
/// Panics if the orientation is [`Orientation::Staggered`] which is not supported by this plugin.
pub fn tilemap_type_from_map(map: &tiled::Map) -> TilemapType {
    match map.orientation {
        tiled::Orientation::Orthogonal => TilemapType::Square,
        tiled::Orientation::Hexagonal => match map.stagger_axis {
            tiled::StaggerAxis::X if map.stagger_index == tiled::StaggerIndex::Even => {
                TilemapType::Hexagon(HexCoordSystem::ColumnOdd)
            }
            tiled::StaggerAxis::X if map.stagger_index == tiled::StaggerIndex::Odd => {
                TilemapType::Hexagon(HexCoordSystem::ColumnEven)
            }
            tiled::StaggerAxis::Y if map.stagger_index == tiled::StaggerIndex::Even => {
                TilemapType::Hexagon(HexCoordSystem::RowOdd)
            }
            tiled::StaggerAxis::Y if map.stagger_index == tiled::StaggerIndex::Odd => {
                TilemapType::Hexagon(HexCoordSystem::RowEven)
            }
            _ => unreachable!(),
        },
        tiled::Orientation::Isometric => TilemapType::Isometric(IsoCoordSystem::Diamond),
        tiled::Orientation::Staggered => {
            panic!("Isometric (Staggered) map is not supported");
        }
    }
}

/// Converts a [`Map`]'s grid size to a [`TilemapGridSize`].
pub fn grid_size_from_map(map: &tiled::Map) -> TilemapGridSize {
    TilemapGridSize {
        x: map.tile_width as f32,
        y: map.tile_height as f32,
    }
}

/// Get the [`TilemapTileSize`] from given [`Tile`]
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
