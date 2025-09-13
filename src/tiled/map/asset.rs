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
    /// Key is a unique label to identify the Tiled tileset within the map.
    /// See [`tileset_label`](crate::tiled::map::loader::tileset_label) function.
    pub(crate) tilesets: HashMap<String, TiledMapTileset>,
    /// HashMap of the label to tilesets
    ///
    /// Key is the Tiled tileset index
    pub(crate) tilesets_label_by_index: HashMap<u32, String>,
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

    /// Iterates over all tiles in the given [`TileLayer`], invoking a callback for each tile.
    ///
    /// This function abstracts over both finite and infinite Tiled map layers, providing a unified
    /// way to visit every tile in a layer. For each tile, the provided closure is called with:
    /// - the [`LayerTile`] (tile instance)
    /// - a reference to the [`LayerTileData`] (tile metadata)
    /// - the [`TilePos`] (tile position in Bevy coordinates)
    /// - the [`IVec2`] (tile position in Tiled chunk coordinates)
    ///
    /// The coordinate conversion ensures that the tile positions are consistent with Bevy's coordinate system,
    /// including Y-axis inversion and chunk offset handling for infinite maps.
    ///
    /// # Arguments
    /// * `tiles_layer` - The Tiled tile layer to iterate over (finite or infinite).
    /// * `f` - A closure to call for each tile, with signature:
    ///   `(LayerTile, &LayerTileData, TilePos, IVec2)`
    ///
    /// # Example
    /// ```rust,no_run
    /// use bevy_ecs_tiled::prelude::*;
    ///
    /// fn print_tile_positions(asset: &TiledMapAsset, layer: &TileLayer) {
    ///     asset.for_each_tile(layer, |tile, data, tile_pos, chunk_pos| {
    ///         println!("Tile at Bevy pos: {:?}, chunk pos: {:?}", tile_pos, chunk_pos);
    ///     });
    /// }
    /// ```
    ///
    /// # Notes
    /// - For infinite maps, chunk positions are shifted so that the top-left chunk is at (0, 0),
    ///   and negative tile coordinates are avoided.
    /// - The Y coordinate is inverted to match Bevy's coordinate system (origin at bottom-left).
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

    /// Returns the world position of a Tiled [`Object`] relative to its parent [`TiledLayer::Objects`] entity.
    ///
    /// The returned position corresponds to the object's top-left anchor in world coordinates, taking into account
    /// the map's anchor, grid size, and coordinate system. This is equivalent to using the object's [`Transform`] component
    /// to get its position, but is provided for convenience and consistency with other Tiled coordinate conversions.
    ///
    /// # Arguments
    /// * `object` - The Tiled object whose position to compute.
    /// * `anchor` - The [`TilemapAnchor`] used for the map.
    ///
    /// # Returns
    /// * `Vec2` - The object's world position relative to its parent layer entity.
    pub fn object_relative_position(&self, object: &Object, anchor: &TilemapAnchor) -> Vec2 {
        self.world_space_from_tiled_position(anchor, Vec2::new(object.x, object.y))
    }

    /// Returns the world position (center) of a tile relative to its parent [`TiledTilemap`] [`Entity`].
    ///
    /// This function computes the world-space position of a tile, given its [`TilePos`], tile size, and map anchor.
    /// It is especially useful because tiles do not have their own [`Transform`] component, so their world position must be calculated manually.
    ///
    /// The returned position is the center of the tile in world coordinates, taking into account the map's size,
    /// grid size, tile size, map type (orthogonal, isometric, hex), and anchor. This ensures correct placement
    /// regardless of map orientation or coordinate system.
    ///
    /// # Arguments
    /// * `tile_pos` - The tile's position in Bevy tile coordinates (origin at bottom-left).
    /// * `tile_size` - The size of the tile in pixels.
    /// * `anchor` - The [`TilemapAnchor`] used for the map.
    ///
    /// # Returns
    /// * `Vec2` - The world-space position of the tile's center, relative to its parent tilemap entity.
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
    ///     for (tiled_map, storage, anchor) in map_query.iter() {
    ///         let Some(map_asset) = assets.get(&tiled_map.0) else { continue; };
    ///         for (_, entities) in storage.tiles() {
    ///             for entity in entities {
    ///                 let Ok((_, tile_pos, child_of)) = tile_query.get(*entity) else { continue; };
    ///                 let Some(tile) = storage.get_tile(&map_asset.map, *entity) else { continue; };
    ///                 // Compute the tile's world position (center)
    ///                 let tile_rel_pos = map_asset.tile_relative_position(
    ///                     tile_pos,
    ///                     &tile_size(&tile),
    ///                     anchor
    ///                 );
    ///                 // To get the tile's global transform, combine with the parent tilemap's transform
    ///                 let Ok(parent_transform) = tilemap_query.get(child_of.parent()) else { continue; };
    ///                 let tile_transform = *parent_transform * Transform::from_translation(tile_rel_pos.extend(0.));
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - The returned position is relative to the parent tilemap entity, not global coordinates.
    /// - For global/world coordinates, combine with the parent tilemap's [`GlobalTransform`].
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
