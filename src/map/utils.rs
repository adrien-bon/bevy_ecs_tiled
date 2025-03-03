//! This module contains utilities functions.
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::{ChunkData, LayerTile, LayerTileData, Map, TileLayer};

use super::TiledMap;

/// Convert a [Map]'s [tiled::Orientation] to a [TilemapType]
pub fn get_map_type(map: &Map) -> TilemapType {
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

/// Convert a [Map]'s grid size to a [TilemapGridSize]
pub fn get_grid_size(map: &Map) -> TilemapGridSize {
    TilemapGridSize {
        x: map.tile_width as f32,
        y: map.tile_height as f32,
    }
}

/// Convert a position from Tiled space to world space.
pub fn from_tiled_position_to_world_space(tiled_map: &TiledMap, tiled_position: Vec2) -> Vec2 {
    let map_size = tiled_map.tilemap_size;
    let map_height = tiled_map.rect.height();
    let grid_size = get_grid_size(&tiled_map.map);
    match get_map_type(&tiled_map.map) {
        TilemapType::Square => {
            tiled_map.tiled_offset
                + Vec2 {
                    x: tiled_position.x,
                    y: map_height - tiled_position.y,
                }
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
            tiled_map.tiled_offset
                + Vec2 {
                    x: tiled_position.x,
                    y: map_height + grid_size.y / 2. - tiled_position.y,
                }
        }
        TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
            tiled_map.tiled_offset
                + Vec2 {
                    x: tiled_position.x,
                    y: map_height - tiled_position.y,
                }
        }
        TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
            tiled_map.tiled_offset
                + Vec2 {
                    x: tiled_position.x,
                    y: map_height + grid_size.y / 4. - tiled_position.y,
                }
        }
        TilemapType::Hexagon(HexCoordSystem::RowEven) => {
            tiled_map.tiled_offset
                + Vec2 {
                    x: tiled_position.x - grid_size.x / 2.,
                    y: map_height + grid_size.y / 4. - tiled_position.y,
                }
        }
        TilemapType::Isometric(IsoCoordSystem::Diamond) => {
            let position = iso_projection(
                tiled_position + tiled_map.tiled_offset,
                &map_size,
                &grid_size,
            );
            Vec2 {
                x: position.x,
                y: map_height / 2. - grid_size.y / 2. - position.y,
            }
        }
        TilemapType::Isometric(IsoCoordSystem::Staggered) => {
            panic!("Isometric (Staggered) map is not supported");
        }
        _ => unreachable!(),
    }
}

/// Iterate over all tiles from the given [TileLayer]
pub fn for_each_tile<'a, F>(tiled_map: &'a TiledMap, tiles_layer: &TileLayer<'a>, mut f: F)
where
    F: FnMut(LayerTile<'a>, &LayerTileData, TilePos, IVec2),
{
    let tilemap_size = tiled_map.tilemap_size;
    match tiles_layer {
        TileLayer::Finite(layer) => {
            for x in 0..tilemap_size.x {
                for y in 0..tilemap_size.y {
                    // Transform TMX coords into bevy coords.
                    let mapped_y = tilemap_size.y - 1 - y;
                    let mapped_x = x as i32;
                    let mapped_y = mapped_y as i32;

                    let Some(layer_tile) = layer.get_tile(mapped_x, mapped_y) else {
                        continue;
                    };
                    let Some(layer_tile_data) = layer.get_tile_data(mapped_x, mapped_y) else {
                        continue;
                    };

                    f(
                        layer_tile,
                        layer_tile_data,
                        TilePos::new(x, y),
                        IVec2::new(mapped_x, mapped_y),
                    );
                }
            }
        }
        TileLayer::Infinite(layer) => {
            for (chunk_pos, chunk) in layer.chunks() {
                // bevy_ecs_tilemap doesn't support negative tile coordinates, so shift all chunks
                // such that the top-left chunk is at (0, 0).
                let chunk_pos_mapped = (
                    chunk_pos.0 - tiled_map.topleft_chunk.0,
                    chunk_pos.1 - tiled_map.topleft_chunk.1,
                );

                for x in 0..ChunkData::WIDTH {
                    for y in 0..ChunkData::HEIGHT {
                        // Invert y to match bevy coordinates.
                        let Some(layer_tile) = chunk.get_tile(x as i32, y as i32) else {
                            continue;
                        };
                        let Some(layer_tile_data) = chunk.get_tile_data(x as i32, y as i32) else {
                            continue;
                        };

                        let index = IVec2 {
                            x: chunk_pos_mapped.0 * ChunkData::WIDTH as i32 + x as i32,
                            y: chunk_pos_mapped.1 * ChunkData::HEIGHT as i32 + y as i32,
                        };

                        f(
                            layer_tile,
                            layer_tile_data,
                            TilePos {
                                x: index.x as u32,
                                y: tilemap_size.y - 1 - index.y as u32,
                            },
                            index,
                        );
                    }
                }
            }
        }
    }
}

/// Convert Tiled isometric coordinates into scalar coordinates
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
