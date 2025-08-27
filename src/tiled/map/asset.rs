//! Asset types and asset loader for Tiled maps.
//!
//! This module defines asset structures, asset events, and the asset loader implementation for importing Tiled maps
//! into Bevy's asset system.

use crate::{prelude::*, tiled::helpers::iso_projection};
use bevy::prelude::*;
use bevy_ecs_tilemap::map::{HexCoordSystem, IsoCoordSystem, TilemapTexture};
use std::fmt;
use tiled::ChunkData;

#[derive(Default, Debug)]
pub(crate) struct TiledMapTileset {
    /// Does this tileset can be used for tiles layer ?
    ///
    /// A tileset can be used for tiles layer only if all the images it contains have the
    /// same dimensions (restriction from bevy_ecs_tilemap).
    pub(crate) usable_for_tiles_layer: bool,
    /// Tileset texture (ie. a single image or an images collection)
    pub(crate) tilemap_texture: TilemapTexture,
    /// The [`TextureAtlasLayout`] handle associated to each tileset, if any.
    pub(crate) texture_atlas_layout_handle: Option<Handle<TextureAtlasLayout>>,
    /// The offset into the tileset_images for each tile id within each tileset.
    #[cfg(not(feature = "atlas"))]
    pub(crate) tile_image_offsets: HashMap<tiled::TileId, u32>,
}

/// Tiled map [`Asset`].
///
/// [`Asset`] holding Tiled map informations.
#[derive(TypePath, Asset)]
pub struct TiledMapAsset {
    /// The raw Tiled map data
    pub map: tiled::Map,
    /// Map size in tiles
    pub tilemap_size: TilemapSize,
    /// The largest tile size, in pixels, found among all tiles from this map
    ///
    /// You should only rely on this value for "map-level" concerns.
    /// If you want to get the actual size of a given tile, you should instead
    /// use the [`tile_size`] function.
    pub largest_tile_size: TilemapTileSize,
    /// Map bounding box, unanchored, in pixels.
    ///
    /// Origin it map bottom-left.
    /// Minimum is `(0., 0.)`
    /// Maximum is `(map_size.x, map_size.y)`
    pub rect: Rect,
    /// Offset to apply on Tiled coordinates when converting to Bevy coordinates
    ///
    /// Our computations for converting coordinates assume that Tiled origin (ie. point [0, 0])
    /// is always in the top-left corner of the map. This is not the case for infinite maps where
    /// map origin is at the top-left corner of chunk (0, 0) and we can have chunk with negative indexes
    pub(crate) tiled_offset: Vec2,
    /// Top-left chunk index
    ///
    /// For finite maps, always (0, 0)
    pub(crate) topleft_chunk: (i32, i32),
    /// Bottom-right chunk index
    ///
    /// For finite maps, always (0, 0)
    pub(crate) bottomright_chunk: (i32, i32),
    /// HashMap of the map tilesets
    ///
    /// Key is the path to the Tiled tileset
    pub(crate) tilesets: HashMap<String, TiledMapTileset>,
    /// HashMap of the paths to tilesets
    ///
    /// Key is the Tiled tileset index
    pub(crate) tilesets_path_by_index: HashMap<u32, String>,
    /// HashMap of the images used in the map
    ///
    /// Key is the layer id of the image layer using this image
    pub(crate) images: HashMap<u32, Handle<Image>>,
    /// Map properties
    #[cfg(feature = "user_properties")]
    pub(crate) properties: crate::tiled::properties::load::DeserializedMapProperties,
}

impl TiledMapAsset {
    /// Convert a position from Tiled space to world space
    pub(crate) fn world_space_from_tiled_position(
        &self,
        anchor: &TilemapAnchor,
        tiled_position: Vec2,
    ) -> Vec2 {
        let map_size = self.tilemap_size;
        let tile_size = self.largest_tile_size;
        let map_height = self.rect.height();
        let grid_size = grid_size_from_map(&self.map);
        let map_type = tilemap_type_from_map(&self.map);
        let mut offset = anchor.as_offset(&map_size, &grid_size, &tile_size, &map_type);
        offset.x -= grid_size.x / 2.0;
        offset.y -= grid_size.y / 2.0;
        offset
            + match map_type {
                TilemapType::Square => {
                    self.tiled_offset
                        + Vec2 {
                            x: tiled_position.x,
                            y: map_height - tiled_position.y,
                        }
                }
                TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => {
                    self.tiled_offset
                        + Vec2 {
                            x: tiled_position.x,
                            y: map_height + grid_size.y / 2. - tiled_position.y,
                        }
                }
                TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
                    self.tiled_offset
                        + Vec2 {
                            x: tiled_position.x,
                            y: map_height - tiled_position.y,
                        }
                }
                TilemapType::Hexagon(HexCoordSystem::RowOdd) => {
                    self.tiled_offset
                        + Vec2 {
                            x: tiled_position.x,
                            y: map_height + grid_size.y / 4. - tiled_position.y,
                        }
                }
                TilemapType::Hexagon(HexCoordSystem::RowEven) => {
                    self.tiled_offset
                        + Vec2 {
                            x: tiled_position.x - grid_size.x / 2.,
                            y: map_height + grid_size.y / 4. - tiled_position.y,
                        }
                }
                TilemapType::Isometric(IsoCoordSystem::Diamond) => {
                    let position =
                        iso_projection(tiled_position + self.tiled_offset, &map_size, &grid_size);
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

    /// Iterate over all tiles from the given [`TileLayer`].
    pub fn for_each_tile<'a, F>(&'a self, tiles_layer: &TileLayer<'a>, mut f: F)
    where
        F: FnMut(LayerTile<'a>, &LayerTileData, TilePos, IVec2),
    {
        let tilemap_size = self.tilemap_size;
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
                        chunk_pos.0 - self.topleft_chunk.0,
                        chunk_pos.1 - self.topleft_chunk.1,
                    );

                    for x in 0..ChunkData::WIDTH {
                        for y in 0..ChunkData::HEIGHT {
                            // Invert y to match bevy coordinates.
                            let Some(layer_tile) = chunk.get_tile(x as i32, y as i32) else {
                                continue;
                            };
                            let Some(layer_tile_data) = chunk.get_tile_data(x as i32, y as i32)
                            else {
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

    /// Retrieve an [`Object`] world position (object top left) relative to its parent [`TiledLayer::Objects`] [`Entity`].
    ///
    /// In most cases it's easier to use the object [`Transform`] component to get its position.
    pub fn object_relative_position(&self, object: &Object, anchor: &TilemapAnchor) -> Vec2 {
        self.world_space_from_tiled_position(anchor, Vec2::new(object.x, object.y))
    }

    /// Retrieve a [`Tile`] world position (tile center) relative to its parent [`TiledTilemap`] [`Entity`].
    ///
    /// This can be useful since tiles do not have a [`Transform`] component.
    ///
    /// # Example
    /// ```rust,no_run
    /// use bevy::prelude::*;
    /// use bevy_ecs_tiled::prelude::*;
    ///
    /// fn tile_position_system(
    ///     assets: Res<Assets<TiledMapAsset>>,
    ///     map_query: Query<(&TiledMap, &TiledMapStorage, &TilemapAnchor)>,
    ///     tile_query: Query<(Entity, &TilePos, &ChildOf), With<TiledTile>>,
    ///     tilemap_query: Query<&GlobalTransform, With<TiledTilemap>>,
    /// ) {
    ///     // We will iterate over tiles using the TiledMapStorage component
    ///     for (tiled_map, storage, anchor) in map_query.iter() {
    ///         let Some(map_asset) = assets.get(&tiled_map.0) else {
    ///             continue;
    ///         };
    ///         for (_, entities) in storage.tiles() {
    ///             for entity in entities {
    ///                 let Ok((entity, tile_pos, child_of)) = tile_query.get(*entity) else {
    ///                     continue;
    ///                 };
    ///                 let Some(tile) = storage.get_tile(&map_asset.map, entity) else {
    ///                     continue;
    ///                 };
    ///
    ///                 // Retrieve the tile relative position
    ///                 let tile_rel_pos = map_asset.tile_relative_position(
    ///                     tile_pos,
    ///                     &tile_size(&tile),
    ///                     anchor
    ///                 );
    ///
    ///                 // If we want to get the tile GlobalTransform,
    ///                 // we need to offset this with GlobalTransform from its parent tilemap
    ///                 let Ok(parent_transform) = tilemap_query.get(child_of.parent()) else {
    ///                     continue;
    ///                 };
    ///                 let tile_transform =
    ///                     *parent_transform * Transform::from_translation(tile_rel_pos.extend(0.));
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn tile_relative_position(
        &self,
        tile_pos: &TilePos,
        tile_size: &TilemapTileSize,
        anchor: &TilemapAnchor,
    ) -> Vec2 {
        tile_pos.center_in_world(
            &self.tilemap_size,
            &grid_size_from_map(&self.map),
            tile_size,
            &tilemap_type_from_map(&self.map),
            anchor,
        )
    }
}

impl fmt::Debug for TiledMapAsset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TiledMapAsset")
            .field("map", &self.map)
            .field("tilemap_size", &self.tilemap_size)
            .field("largest_tile_size", &self.largest_tile_size)
            .field("rect", &self.rect)
            .field("tiled_offset", &self.tiled_offset)
            .field("topleft_chunk", &self.topleft_chunk)
            .field("bottomright_chunk", &self.bottomright_chunk)
            .finish()
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_asset::<TiledMapAsset>();
}
