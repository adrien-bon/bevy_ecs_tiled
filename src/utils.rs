//! This module contains utilities functions.
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::Map;

/// Convert a [tiled::Map]'s [tiled::Orientation] to a [TilemapType]
pub fn get_map_type(map: &Map) -> TilemapType {
    match map.orientation {
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
            warn!("Isometric (Staggered) map is not supported");
            TilemapType::Isometric(IsoCoordSystem::Staggered)
        }
        tiled::Orientation::Orthogonal => TilemapType::Square,
    }
}

/// Convert a [tiled::Map]'s size to a [TilemapSize]
pub fn get_map_size(map: &Map) -> TilemapSize {
    TilemapSize {
        x: map.width,
        y: map.height,
    }
}

/// Convert a [tiled::Map]'s grid size to a [TilemapGridSize]
pub fn get_grid_size(map: &Map) -> TilemapGridSize {
    TilemapGridSize {
        x: map.tile_width as f32,
        y: map.tile_height as f32,
    }
}

/// Convert from Tiled coordinates to a Bevy position.
///
/// This function will convert provided Tiled raw position to a Bevy position, according to various maps settings.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
/// use bevy_ecs_tilemap::prelude::*;
///
/// let tiled_position = Vec2::new(0., 12.);
/// let bevy_position = from_tiled_coords_to_bevy(
///     tiled_position,
///     &TilemapType::Square,
///     &TilemapSize::new(0, 3),
///     &TilemapGridSize::new(16., 16.),
/// );
/// ```
pub fn from_tiled_coords_to_bevy(
    tiled_position: Vec2,
    map_type: &TilemapType,
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
) -> Vec2 {
    match map_type {
        TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => Vec2::new(
            tiled_position.x + grid_size.x / 4.,
            (map_size.y as f32 + 0.5) * grid_size.y - tiled_position.y,
        ),
        TilemapType::Hexagon(HexCoordSystem::ColumnEven) => Vec2::new(
            tiled_position.x + grid_size.x / 4.,
            (map_size.y as f32 + 0.) * grid_size.y - tiled_position.y,
        ),
        TilemapType::Hexagon(HexCoordSystem::RowOdd) => Vec2::new(
            tiled_position.x,
            map_size.y as f32 * grid_size.y * 0.75 + grid_size.y / 4. - tiled_position.y,
        ),
        TilemapType::Hexagon(HexCoordSystem::RowEven) => Vec2::new(
            tiled_position.x - grid_size.x / 2.,
            map_size.y as f32 * grid_size.y * 0.75 + grid_size.y / 4. - tiled_position.y,
        ),
        TilemapType::Isometric(coords_system) => {
            from_isometric_coords_to_bevy(tiled_position, coords_system, map_size, grid_size)
        }
        _ => Vec2::new(
            tiled_position.x,
            map_size.y as f32 * grid_size.y - tiled_position.y,
        ),
    }
}

/// Convert from Tiled isometric coordinates to a Bevy position.
///
/// This function will convert provided Tiled raw isometric position to a Bevy position, according to various maps settings.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
/// use bevy_ecs_tilemap::prelude::*;
///
/// let tiled_position = Vec2::new(0., 12.);
/// let bevy_position = from_isometric_coords_to_bevy(
///     tiled_position,
///     &IsoCoordSystem::Diamond,
///     &TilemapSize::new(0, 3),
///     &TilemapGridSize::new(16., 16.),
/// );
/// ```
pub fn from_isometric_coords_to_bevy(
    tiled_position: Vec2,
    iso_coords: &IsoCoordSystem,
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
) -> Vec2 {
    match iso_coords {
        IsoCoordSystem::Diamond => Vec2::new(
            ((tiled_position.x - tiled_position.y) / grid_size.y + map_size.y as f32) * grid_size.x
                / 2.,
            (map_size.y as f32
                - tiled_position.x / grid_size.y
                - tiled_position.y / grid_size.y
                - 1.)
                * grid_size.y
                / 2.,
        ),
        IsoCoordSystem::Staggered => {
            //warn!("Isometric (Staggered) map is not supported");
            tiled_position
        }
    }
}
