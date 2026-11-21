//! The Tiled map [`Asset`] and its loader.

mod loader;

pub use loader::TiledMapLoaderError;
pub(crate) use loader::tileset_label;

use crate::{asset::helpers::iso_projection, prelude::*};
use bevy::{platform::collections::HashMap, prelude::*};
use std::{fmt, sync::Arc};

#[derive(Debug)]
pub(crate) struct TiledMapTileset {
    /// The raw Tiled tileset data
    pub(crate) tileset: Arc<tiled::Tileset>,
    /// Textures of the tileset
    #[cfg(feature = "render")]
    pub(crate) textures: crate::render::tileset::TiledTilesetTextures,
}

/// Tiled map [`Asset`].
///
/// [`Asset`] holding Tiled map informations.
#[derive(TypePath, Asset)]
pub struct TiledMapAsset {
    /// The raw Tiled map data
    pub map: tiled::Map,
    /// Map type, converted from the Tiled map orientation.
    ///
    /// Maps with an unsupported orientation are rejected by the map loader.
    pub map_type: TilemapType,
    /// Size of a cell of the map grid, in pixels, as used by `bevy_ecs_tilemap`.
    ///
    /// It is the map tile size, except for hexagonal maps whose `hexsidelength` is not half the
    /// tile size: `bevy_ecs_tilemap` spaces staggered rows (or columns) by `0.75 * grid_size`, so
    /// the grid size is adjusted along the stagger axis to match Tiled spacing, which is
    /// `(tile_size + hexsidelength) / 2`.
    /// Use this value with `bevy_ecs_tilemap` helpers, such as [`TilePos::center_in_world`].
    pub grid_size: TilemapGridSize,
    /// Map size in tiles
    pub tilemap_size: TilemapSize,
    /// The largest tile size, in pixels, found among all tiles from this map
    ///
    /// You should only rely on this value for "map-level" concerns.
    /// If you want to get the actual size of a given tile, you should instead
    /// use [`TiledMapAsset::tile_size`].
    pub largest_tile_size: TilemapTileSize,
    /// Map bounding box, unanchored, in pixels.
    ///
    /// Origin is map bottom-left.
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
    /// Map of the map tilesets
    ///
    /// Key is the [`TilesetId`].
    pub(crate) tilesets: HashMap<TilesetId, TiledMapTileset>,
    /// Map of the tilesets ID by label
    ///
    /// Key is a unique label to identify the Tiled tileset within the map.
    /// See [`tileset_label`] function.
    pub(crate) tilesets_id_by_label: HashMap<String, TilesetId>,
    /// Map of the images used in the map
    ///
    /// Key is the [`LayerId`] of the image layer using this image
    #[cfg(feature = "render")]
    pub(crate) images: HashMap<LayerId, Handle<Image>>,
    /// Map properties
    #[cfg(feature = "user_properties")]
    pub(crate) properties: crate::properties::user::load::DeserializedMapProperties,
}

impl TiledMapAsset {
    /// Retrieve the [`tiled::Tileset`] associated with this [`TilesetId`].
    ///
    /// It also works for tilesets only referenced by templates, which are not part of [`tiled::Map::tilesets`].
    pub fn tileset(&self, tileset_id: TilesetId) -> Option<&Arc<tiled::Tileset>> {
        self.tilesets.get(&tileset_id).map(|t| &t.tileset)
    }

    /// Retrieve the [`tiled::Tile`] associated with this [`TilesetId`] and [`tiled::TileId`].
    ///
    /// It also works for tilesets only referenced by templates, which are not part of [`tiled::Map::tilesets`].
    pub fn tile(&self, tileset_id: TilesetId, tile_id: tiled::TileId) -> Option<tiled::Tile<'_>> {
        self.tileset(tileset_id).and_then(|t| t.get_tile(tile_id))
    }

    /// Retrieve the [`tiled::Layer`] associated with this [`LayerId`], whatever its type.
    ///
    /// Layers nested in group layers are found as well.
    pub fn layer(&self, layer_id: LayerId) -> Option<tiled::Layer<'_>> {
        crate::asset::helpers::layer_from_map(&self.map, layer_id)
    }

    /// Retrieve the [`tiled::Object`] associated with this [`ObjectId`].
    ///
    /// All objects layers are searched, including the ones nested in group layers.
    pub fn object(&self, object_id: ObjectId) -> Option<tiled::Object<'_>> {
        crate::asset::helpers::object_from_map(&self.map, object_id)
    }

    /// Position of Tiled "screen" origin, relative to the map entity.
    ///
    /// This is the point from which Tiled image layers are positioned (before applying the layer offset).
    /// For orthogonal and hexagonal maps, this is Tiled position `(0, 0)`.
    /// For isometric maps, Tiled renderer puts this origin `map.height * tile_width / 2` pixels on the left
    /// of tile `(0, 0)` top corner, where `map.height` is the map height declared in the Tiled map
    /// (which can differ from [`TiledMapAsset::tilemap_size`] for infinite maps).
    pub(crate) fn screen_origin(&self, anchor: &TilemapAnchor) -> Vec2 {
        self.world_space_from_tiled_position(
            anchor,
            match self.map_type {
                // In Tiled isometric coordinates, moving by `(-h/2, h/2)` tiles
                // results in an horizontal move of `-h * tile_width / 2` pixels
                TilemapType::Isometric(IsoCoordSystem::Diamond) => {
                    let grid_size = self.grid_size;
                    let half_height = self.map.height as f32 * grid_size.y / 2.;
                    Vec2::new(-half_height, half_height)
                }
                _ => Vec2::ZERO,
            },
        )
    }

    /// Raw conversion between a Tiled position and Bevy world space
    pub fn world_space_from_tiled_position(
        &self,
        anchor: &TilemapAnchor,
        tiled_position: Vec2,
    ) -> Vec2 {
        let map_size = self.tilemap_size;
        let tile_size = self.largest_tile_size;
        let map_height = self.rect.height();
        let grid_size = self.grid_size;
        let map_type = self.map_type;
        let mut offset = anchor.as_offset(&map_size, &grid_size, &tile_size, &map_type);
        // Move from `bevy_ecs_tilemap` tile (0, 0) center to its bottom-left corner.
        // Use Tiled cell size: for hexagonal maps, `grid_size` can differ from it along
        // the stagger axis (see `TiledMapAsset::grid_size`)
        offset.x -= self.map.tile_width as f32 / 2.0;
        offset.y -= self.map.tile_height as f32 / 2.0;
        offset
            + match map_type {
                TilemapType::Isometric(_) => {
                    let position =
                        iso_projection(tiled_position + self.tiled_offset, &map_size, &grid_size);
                    // Tiled tile (0, 0) top corner is located half a tile above
                    // `bevy_ecs_tilemap` tile (0, map_size.y - 1) center
                    Vec2 {
                        x: position.x,
                        y: (map_size.y + 1) as f32 * grid_size.y / 2. - position.y,
                    }
                }
                TilemapType::Square | TilemapType::Hexagon(_) => {
                    // Hexagonal maps whose first row / column is staggered are shifted by half a cell
                    let stagger = match map_type {
                        TilemapType::Hexagon(HexCoordSystem::ColumnEven) => {
                            Vec2::new(0., -grid_size.y / 2.)
                        }
                        TilemapType::Hexagon(HexCoordSystem::RowEven) => {
                            Vec2::new(-grid_size.x / 2., 0.)
                        }
                        _ => Vec2::ZERO,
                    };
                    self.tiled_offset
                        + stagger
                        + Vec2 {
                            x: tiled_position.x,
                            y: map_height - tiled_position.y,
                        }
                }
            }
    }

    /// Iterates over all tiles in the given [`tiled::TileLayer`], invoking a callback for each tile.
    ///
    /// This function abstracts over both finite and infinite Tiled map layers, providing a unified
    /// way to visit every tile in a layer. For each tile, the provided closure is called with:
    /// - the [`tiled::LayerTile`] (tile instance)
    /// - a reference to the [`tiled::LayerTileData`] (tile metadata)
    /// - the [`TilePos`] (tile position in Bevy coordinates)
    /// - the [`IVec2`] (tile position in Tiled chunk coordinates)
    ///
    /// The coordinate conversion ensures that the tile positions are consistent with Bevy's coordinate system,
    /// including Y-axis inversion and chunk offset handling for infinite maps.
    ///
    /// # Example
    /// ```rust,no_run
    /// use bevy_ecs_tiled::{prelude::*, tiled};
    ///
    /// fn print_tile_positions(asset: &TiledMapAsset, layer: &tiled::TileLayer) {
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
    pub fn for_each_tile<'a, F>(&'a self, tiles_layer: &tiled::TileLayer<'a>, mut f: F)
    where
        F: FnMut(tiled::LayerTile<'a>, &tiled::LayerTileData, TilePos, IVec2),
    {
        let tilemap_size = self.tilemap_size;
        match tiles_layer {
            tiled::TileLayer::Finite(layer) => {
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
            tiled::TileLayer::Infinite(layer) => {
                for (chunk_pos, chunk) in layer.chunks() {
                    // bevy_ecs_tilemap doesn't support negative tile coordinates, so shift all chunks
                    // such that the top-left chunk is at (0, 0).
                    let chunk_pos_mapped = (
                        chunk_pos.0 - self.topleft_chunk.0,
                        chunk_pos.1 - self.topleft_chunk.1,
                    );

                    for x in 0..tiled::ChunkData::WIDTH {
                        for y in 0..tiled::ChunkData::HEIGHT {
                            // Invert y to match bevy coordinates.
                            let Some(layer_tile) = chunk.get_tile(x as i32, y as i32) else {
                                continue;
                            };
                            let Some(layer_tile_data) = chunk.get_tile_data(x as i32, y as i32)
                            else {
                                continue;
                            };

                            let index = IVec2 {
                                x: chunk_pos_mapped.0 * tiled::ChunkData::WIDTH as i32 + x as i32,
                                y: chunk_pos_mapped.1 * tiled::ChunkData::HEIGHT as i32 + y as i32,
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

    /// Returns the world position of a Tiled [`tiled::Object`] relative to its parent [`TiledLayer::Objects`] entity.
    ///
    /// The returned position corresponds to the object's top-left anchor in world coordinates, taking into account
    /// the map's anchor, grid size, and coordinate system. This is equivalent to using the object's [`Transform`] component
    /// to get its position, but is provided for convenience and consistency with other Tiled coordinate conversions.
    pub fn object_relative_position(&self, object: &tiled::Object, anchor: &TilemapAnchor) -> Vec2 {
        self.world_space_from_tiled_position(anchor, Vec2::new(object.x, object.y))
    }

    /// Returns the center of a Tiled object in world coordinates, taking into account map type and transform.
    pub fn object_center(
        &self,
        tiled_object: &TiledObject,
        transform: &GlobalTransform,
    ) -> Option<geo::Coord<f32>> {
        tiled_object.center(
            transform,
            matches!(self.map_type, TilemapType::Isometric(..)),
            &self.tilemap_size,
            &self.grid_size,
            self.tiled_offset,
        )
    }

    /// Returns the vertices of a Tiled object in world coordinates, taking into account map type and transform.
    pub fn object_vertices(
        &self,
        tiled_object: &TiledObject,
        transform: &GlobalTransform,
    ) -> Vec<geo::Coord<f32>> {
        tiled_object.vertices(
            transform,
            matches!(self.map_type, TilemapType::Isometric(..)),
            &self.tilemap_size,
            &self.grid_size,
            self.tiled_offset,
        )
    }

    /// Returns the object's geometry as a [`geo::LineString`] in world coordinates, if applicable.
    pub fn object_line_string(
        &self,
        tiled_object: &TiledObject,
        transform: &GlobalTransform,
    ) -> Option<geo::LineString<f32>> {
        tiled_object.line_string(
            transform,
            matches!(self.map_type, TilemapType::Isometric(..)),
            &self.tilemap_size,
            &self.grid_size,
            self.tiled_offset,
        )
    }

    /// Returns the object's geometry as a [`geo::Polygon`] in world coordinates, if applicable.
    pub fn object_polygon(
        &self,
        tiled_object: &TiledObject,
        transform: &GlobalTransform,
    ) -> Option<geo::Polygon<f32>> {
        tiled_object.polygon(
            transform,
            matches!(self.map_type, TilemapType::Isometric(..)),
            &self.tilemap_size,
            &self.grid_size,
            self.tiled_offset,
        )
    }

    /// Size, in pixels, of the image of the given tile.
    ///
    /// It is the tileset tile size, except for tiles of images collection tilesets, which have their own image size.
    /// Returns [`None`] if the tile does not exist in this map.
    pub fn tile_size(&self, tile: &TiledTile) -> Option<TilemapTileSize> {
        self.tile(tile.tileset_id, tile.tile_id)
            .map(|tile| crate::asset::helpers::tile_size(&tile))
    }

    /// Returns the position of the center of a tile, relative to its [`TiledTilemap`] [`Entity`].
    ///
    /// Tiles do not have a [`Transform`]: use this method to know where a tile is rendered.
    /// It takes into account the map size, grid size, orientation and anchor, and the size of the tile image.
    ///
    /// Returns [`None`] if the tile does not exist in this map.
    ///
    /// Example:
    /// ```rust,no_run
    /// # #[cfg(feature = "render")]
    /// # {
    /// use bevy::prelude::*;
    /// use bevy_ecs_tiled::prelude::*;
    ///
    /// fn tile_position_system(
    ///     assets: Res<Assets<TiledMapAsset>>,
    ///     map_query: Query<(&TiledMap, &TilemapAnchor)>,
    ///     tile_query: Query<(&TilePos, &TiledTile, &TilemapId, &TiledMapReference)>,
    ///     tilemap_query: Query<&GlobalTransform, With<TiledTilemap>>,
    /// ) {
    ///     for (tile_pos, tile, tilemap_id, map_ref) in tile_query.iter() {
    ///         let Ok((tiled_map, anchor)) = map_query.get(map_ref.entity()) else { continue; };
    ///         let Some(map_asset) = assets.get(&tiled_map.0) else { continue; };
    ///         let Some(position) = map_asset.tile_relative_position(tile_pos, tile, anchor) else { continue; };
    ///         // Combine with the tilemap transform to get the tile global position
    ///         let Ok(tilemap_transform) = tilemap_query.get(tilemap_id.0) else { continue; };
    ///         let tile_position = tilemap_transform.transform_point(position.extend(0.));
    ///     }
    /// }
    /// # }
    /// ```
    pub fn tile_relative_position(
        &self,
        tile_pos: &TilePos,
        tile: &TiledTile,
        anchor: &TilemapAnchor,
    ) -> Option<Vec2> {
        Some(self.tile_center(tile_pos, &self.tile_size(tile)?, anchor))
    }

    /// Same as [`TiledMapAsset::tile_relative_position`], for a tile whose image size is already known.
    pub(crate) fn tile_center(
        &self,
        tile_pos: &TilePos,
        tile_size: &TilemapTileSize,
        anchor: &TilemapAnchor,
    ) -> Vec2 {
        tile_pos.center_in_world(
            &self.tilemap_size,
            &self.grid_size,
            tile_size,
            &self.map_type,
            anchor,
        )
    }
}

impl fmt::Debug for TiledMapAsset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TiledMapAsset")
            .field("map", &self.map)
            .field("map_type", &self.map_type)
            .field("grid_size", &self.grid_size)
            .field("tilemap_size", &self.tilemap_size)
            .field("largest_tile_size", &self.largest_tile_size)
            .field("rect", &self.rect)
            .field("tiled_offset", &self.tiled_offset)
            .field("topleft_chunk", &self.topleft_chunk)
            .field("bottomright_chunk", &self.bottomright_chunk)
            .finish_non_exhaustive()
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_asset::<TiledMapAsset>();
    app.add_plugins(loader::plugin);
}
