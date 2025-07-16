//! Internal helper functions and utilities for the bevy_ecs_tiled plugin.
//!
//! This module provides a collection of utility functions used throughout the crate for tasks such as
//! coordinate conversions, data extraction, and other operations related to Tiled maps and worlds.

use std::sync::Arc;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::{HexCoordSystem, IsoCoordSystem};
use tiled::{Orientation, StaggerAxis, StaggerIndex};

/// Retrieves a [`Layer`] from a [`Map`] given a layer ID.
///
/// Returns `Some(Layer)` if the layer exists, or `None` if the ID is out of bounds.
pub fn get_layer_from_map(map: &Map, layer_id: u32) -> Option<Layer> {
    map.get_layer(layer_id as usize)
}

/// Retrieves a [`Tileset`] from a [`Map`] given a tileset ID.
///
/// Returns a reference to the tileset if found, or `None` if the ID is invalid.
pub fn get_tileset_from_map(map: &Map, tileset_id: u32) -> Option<&Arc<Tileset>> {
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
pub fn get_tile_from_map(map: &Map, tileset_id: u32, tile_id: TileId) -> Option<Tile> {
    get_tileset_from_map(map, tileset_id).and_then(|t| t.get_tile(tile_id))
}

/// Retrieves an [`Object`] from a [`Map`] given an object ID.
///
/// Searches all object layers for the specified object ID and returns it if found.
pub fn get_object_from_map(map: &Map, object_id: u32) -> Option<Object> {
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
pub fn tilemap_type_from_map(map: &Map) -> TilemapType {
    match map.orientation {
        Orientation::Orthogonal => TilemapType::Square,
        Orientation::Hexagonal => match map.stagger_axis {
            StaggerAxis::X if map.stagger_index == StaggerIndex::Even => {
                TilemapType::Hexagon(HexCoordSystem::ColumnOdd)
            }
            StaggerAxis::X if map.stagger_index == StaggerIndex::Odd => {
                TilemapType::Hexagon(HexCoordSystem::ColumnEven)
            }
            StaggerAxis::Y if map.stagger_index == StaggerIndex::Even => {
                TilemapType::Hexagon(HexCoordSystem::RowOdd)
            }
            StaggerAxis::Y if map.stagger_index == StaggerIndex::Odd => {
                TilemapType::Hexagon(HexCoordSystem::RowEven)
            }
            _ => unreachable!(),
        },
        Orientation::Isometric => TilemapType::Isometric(IsoCoordSystem::Diamond),
        Orientation::Staggered => {
            panic!("Isometric (Staggered) map is not supported");
        }
    }
}

/// Converts a [`TilemapGridSize`] to a [`TilemapTileSize`].
pub fn tile_size_from_grid_size(grid_size: &TilemapGridSize) -> TilemapTileSize {
    // TODO: Do Tiled files have tile size and grid size in sync always?
    TilemapTileSize {
        x: grid_size.x,
        y: grid_size.y,
    }
}

/// Converts a [`Map`]'s grid size to a [`TilemapGridSize`].
pub fn grid_size_from_map(map: &Map) -> TilemapGridSize {
    TilemapGridSize {
        x: map.tile_width as f32,
        y: map.tile_height as f32,
    }
}

/// Converts a [`Map`]'s grid size to a [`TilemapTileSize`].
///
/// The width and height will be the same as those given by [`grid_size_from_map`].
///
/// **Note:** `bevy_ecs_tiled` assumes tile size and grid size have the same values;
/// `bevy_ecs_tilemap` permits them to be different.
pub fn tile_size_from_map(map: &Map) -> TilemapTileSize {
    TilemapTileSize {
        x: map.tile_width as f32,
        y: map.tile_height as f32,
    }
}

/// Projects Tiled isometric coordinates into scalar coordinates for Bevy.
///
/// Used to convert isometric tile coordinates into world-space positions for rendering.
///
/// # Arguments
/// - `coords`: The isometric coordinates to project.
/// - `tilemap_size`: The size of the tilemap.
/// - `grid_size`: The size of each grid cell.
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
    Vec2 {
        x: tilemap_size.y as f32 * grid_size.x / 2. + (fract.x - fract.y) * grid_size.x / 2.,
        y: (fract.x + fract.y) * grid_size.y / 2.,
    }
}

#[allow(dead_code)]
/// Converts an [`Isometry2d`] to a Bevy [`GlobalTransform`].
pub(crate) fn global_transform_from_isometry_2d(isometry_2d: &Isometry2d) -> GlobalTransform {
    GlobalTransform::from_isometry(Isometry3d {
        rotation: Quat::from_rotation_z(isometry_2d.rotation.as_radians()),
        translation: isometry_2d.translation.extend(0.).into(),
    })
}
