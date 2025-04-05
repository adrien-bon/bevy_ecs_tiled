//! This module contains all map [Asset]s definition.

#[cfg(feature = "user_properties")]
use std::ops::Deref;
use std::{fmt, io::ErrorKind};

#[cfg(feature = "user_properties")]
use bevy::reflect::TypeRegistryArc;
use tiled::ChunkData;

#[cfg(feature = "user_properties")]
use crate::properties::load::DeserializedMapProperties;

use crate::{
    cache::TiledResourceCache, get_grid_size, get_map_type, iso_projection,
    reader::BytesResourceReader,
};

use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, LoadContext}, platform_support::collections::HashMap, prelude::*
};

use bevy_ecs_tilemap::prelude::*;

/// Tiled map [Asset].
///
/// [Asset] holding Tiled map informations.
#[derive(TypePath, Asset)]
pub struct TiledMap {
    /// The raw Tiled map data
    pub map: tiled::Map,
    /// Map size in tiles
    pub tilemap_size: TilemapSize,
    /// Map bounding box, unanchored.
    ///
    /// Origin it map bottom-left.
    /// Minimum is set at `(0., 0.)`
    /// Maximum is set at `(map_size.x, map_size.y)`
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
    /// Key is the Tiled tileset index
    pub(crate) tilesets: HashMap<usize, TiledMapTileset>,
    /// Map properties
    #[cfg(feature = "user_properties")]
    pub(crate) properties: DeserializedMapProperties,
}

pub(crate) fn tile_size_from_grid(grid_size: &TilemapGridSize) -> TilemapTileSize {
    // TODO: Do Tiled files have tile size and grid size in sync always?
    TilemapTileSize {
        x: grid_size.x,
        y: grid_size.y,
    }
}

impl TiledMap {
    /// Offset that should be applied to map underlying layers to account for the [TilemapAnchor]
    pub fn offset(&self, anchor: &TilemapAnchor) -> Vec2 {
        let map_type = get_map_type(&self.map);
        let grid_size = get_grid_size(&self.map);

        // TODO: Do Tiled files have tile size and grid size in sync always? We assume so.
        let tile_size = tile_size_from_grid(&grid_size);
        let mut offset = anchor.as_offset(&self.tilemap_size, &grid_size, &tile_size, &map_type);

        // Special case for isometric maps: bevy_ecs_tilemap start drawing
        // them from middle left instead of from bottom left
        if let TilemapType::Isometric(IsoCoordSystem::Diamond) = map_type {
            offset += Vec2::new(0., self.tilemap_size.y as f32 * grid_size.y as f32 / 2.);
        }

        offset
    }
}

impl fmt::Debug for TiledMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TiledMap")
            .field("map", &self.map)
            .field("tilemap_size", &self.tilemap_size)
            .field("rect", &self.rect)
            .field("tiled_offset", &self.tiled_offset)
            .field("topleft_chunk", &self.topleft_chunk)
            .field("bottomright_chunk", &self.bottomright_chunk)
            .finish()
    }
}

#[derive(Default, Debug)]
pub(crate) struct TiledMapTileset {
    /// Does this tileset can be used for tiles layer ?
    ///
    /// A tileset can be used for tiles layer only if all the images it contains have the
    /// same dimensions (restriction from bevy_ecs_tilemap).
    pub(crate) usable_for_tiles_layer: bool,
    /// Tileset texture (ie. a single image or an images collection)
    pub(crate) tilemap_texture: TilemapTexture,
    /// The [TextureAtlasLayout] handle associated to each tileset, if any.
    pub(crate) texture_atlas_layout_handle: Option<Handle<TextureAtlasLayout>>,
    /// The offset into the tileset_images for each tile id within each tileset.
    #[cfg(not(feature = "atlas"))]
    pub(crate) tile_image_offsets: HashMap<tiled::TileId, u32>,
}

pub(crate) struct TiledMapLoader {
    pub cache: TiledResourceCache,
    #[cfg(feature = "user_properties")]
    pub registry: TypeRegistryArc,
}

impl FromWorld for TiledMapLoader {
    fn from_world(world: &mut World) -> Self {
        Self {
            cache: world.resource::<TiledResourceCache>().clone(),
            #[cfg(feature = "user_properties")]
            registry: world.resource::<AppTypeRegistry>().0.clone(),
        }
    }
}

/// [TiledMap] loading error.
#[derive(Debug, thiserror::Error)]
pub enum TiledMapLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for TiledMapLoader {
    type Asset = TiledMap;
    type Settings = ();
    type Error = TiledMapLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        debug!("Start loading map '{}'", load_context.path().display());

        let map_path = load_context.path().to_path_buf();
        let map = {
            // Allow the loader to also load tileset images.
            let mut loader = tiled::Loader::with_cache_and_reader(
                self.cache.clone(),
                BytesResourceReader::new(&bytes, load_context),
            );
            // Load the map and all tiles.
            loader.load_tmx_map(&map_path).map_err(|e| {
                std::io::Error::new(ErrorKind::Other, format!("Could not load TMX map: {e}"))
            })?
        };

        let mut tilesets = HashMap::default();
        for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
            debug!(
                "Loading tileset (index={:?} name={:?}) from {:?}",
                tileset_index, tileset.name, tileset.source
            );
            let mut texture_atlas_layout_handle = None;
            #[cfg(not(feature = "atlas"))]
            let mut tile_image_offsets = HashMap::default();
            let (usable_for_tiles_layer, tilemap_texture) = match &tileset.image {
                None => {
                    #[cfg(feature = "atlas")]
                    {
                        info!("Skipping image collection tileset '{}' which is incompatible with atlas feature", tileset.name);
                        continue;
                    }

                    #[cfg(not(feature = "atlas"))]
                    {
                        let mut usable_for_tiles_layer = true;
                        let mut image_size: Option<(i32, i32)> = None;
                        let mut tile_images: Vec<Handle<Image>> = Vec::new();
                        for (tile_id, tile) in tileset.tiles() {
                            if let Some(img) = &tile.image {
                                let asset_path = AssetPath::from(img.source.clone());
                                trace!("Loading tile image from {asset_path:?} as image ({tileset_index}, {tile_id})");
                                let texture: Handle<Image> = load_context.load(asset_path.clone());
                                tile_image_offsets.insert(tile_id, tile_images.len() as u32);
                                tile_images.push(texture.clone());
                                if usable_for_tiles_layer {
                                    if let Some(image_size) = image_size {
                                        if img.width != image_size.0 || img.height != image_size.1 {
                                            usable_for_tiles_layer = false;
                                        }
                                    } else {
                                        image_size = Some((img.width, img.height));
                                    }
                                }
                            }
                        }
                        if !usable_for_tiles_layer {
                            debug!(
                                "Tileset (index={:?}) have non constant image size and cannot be used for tiles layer",
                                tileset_index
                            );
                        }
                        (usable_for_tiles_layer, TilemapTexture::Vector(tile_images))
                    }
                }
                Some(img) => {
                    let asset_path = AssetPath::from(img.source.clone());
                    let texture: Handle<Image> = load_context.load(asset_path.clone());

                    let columns = (img.width as u32 - tileset.margin + tileset.spacing)
                        / (tileset.tile_width + tileset.spacing);
                    if columns > 0 {
                        texture_atlas_layout_handle = Some(load_context.labeled_asset_scope(
                            tileset.name.clone(),
                            |_| {
                                TextureAtlasLayout::from_grid(
                                    UVec2::new(tileset.tile_width, tileset.tile_height),
                                    columns,
                                    tileset.tilecount / columns,
                                    Some(UVec2::new(tileset.spacing, tileset.spacing)),
                                    Some(UVec2::new(
                                        tileset.offset_x as u32 + tileset.margin,
                                        tileset.offset_y as u32 + tileset.margin,
                                    )),
                                )
                            }
                        ));
                    }

                    (true, TilemapTexture::Single(texture.clone()))
                }
            };
            tilesets.insert(
                tileset_index,
                TiledMapTileset {
                    usable_for_tiles_layer,
                    tilemap_texture,
                    texture_atlas_layout_handle,
                    #[cfg(not(feature = "atlas"))]
                    tile_image_offsets,
                },
            );
        }

        let mut infinite = false;

        // Determine top left chunk index of all infinite layers for this map
        let mut topleft = (999999, 999999);
        for layer in map.layers() {
            if let tiled::LayerType::Tiles(tiled::TileLayer::Infinite(layer)) = layer.layer_type() {
                topleft = layer.chunks().fold(topleft, |acc, (pos, _)| {
                    (acc.0.min(pos.0), acc.1.min(pos.1))
                });
                infinite = true;
            }
        }
        // Determine bottom right chunk index of all infinite layers for this map
        let mut bottomright = (0, 0);
        for layer in map.layers() {
            if let tiled::LayerType::Tiles(tiled::TileLayer::Infinite(layer)) = layer.layer_type() {
                bottomright = layer.chunks().fold(bottomright, |acc, (pos, _)| {
                    (acc.0.max(pos.0), acc.1.max(pos.1))
                });
                infinite = true;
            }
        }

        let map_type = get_map_type(&map);
        let grid_size = get_grid_size(&map);
        let (tilemap_size, tiled_offset) = if infinite {
            debug!(
                "(infinite map) topleft = {:?}, bottomright = {:?}",
                topleft, bottomright
            );
            (
                TilemapSize {
                    x: (bottomright.0 - topleft.0 + 1) as u32 * ChunkData::WIDTH,
                    y: (bottomright.1 - topleft.1 + 1) as u32 * ChunkData::HEIGHT,
                },
                match map_type {
                    TilemapType::Square => Vec2 {
                        x: -topleft.0 as f32 * ChunkData::WIDTH as f32 * grid_size.x,
                        y: topleft.1 as f32 * ChunkData::HEIGHT as f32 * grid_size.y,
                    },
                    TilemapType::Hexagon(HexCoordSystem::ColumnOdd)
                    | TilemapType::Hexagon(HexCoordSystem::ColumnEven) => Vec2 {
                        x: -topleft.0 as f32 * ChunkData::WIDTH as f32 * grid_size.x * 0.75,
                        y: topleft.1 as f32 * ChunkData::HEIGHT as f32 * grid_size.y,
                    },
                    TilemapType::Hexagon(HexCoordSystem::RowOdd)
                    | TilemapType::Hexagon(HexCoordSystem::RowEven) => Vec2 {
                        x: -topleft.0 as f32 * ChunkData::WIDTH as f32 * grid_size.x,
                        y: topleft.1 as f32 * ChunkData::HEIGHT as f32 * grid_size.y * 0.75,
                    },
                    TilemapType::Isometric(IsoCoordSystem::Diamond) => Vec2 {
                        x: -topleft.0 as f32 * ChunkData::WIDTH as f32 * grid_size.y,
                        y: -topleft.1 as f32 * ChunkData::HEIGHT as f32 * grid_size.y,
                    },
                    TilemapType::Isometric(IsoCoordSystem::Staggered) => {
                        panic!("Isometric (Staggered) map is not supported");
                    }
                    _ => unreachable!(),
                },
            )
        } else {
            topleft = (0, 0);
            bottomright = (0, 0);
            (
                TilemapSize {
                    x: map.width,
                    y: map.height,
                },
                Vec2::ZERO,
            )
        };

        let rect = Rect {
            min: Vec2::ZERO,
            max: match map_type {
                TilemapType::Square => Vec2 {
                    x: tilemap_size.x as f32 * grid_size.x,
                    y: tilemap_size.y as f32 * grid_size.y,
                },
                TilemapType::Hexagon(HexCoordSystem::ColumnOdd)
                | TilemapType::Hexagon(HexCoordSystem::ColumnEven) => Vec2 {
                    x: tilemap_size.x as f32 * grid_size.x * 0.75,
                    y: tilemap_size.y as f32 * grid_size.y,
                },
                TilemapType::Hexagon(HexCoordSystem::RowOdd)
                | TilemapType::Hexagon(HexCoordSystem::RowEven) => Vec2 {
                    x: tilemap_size.x as f32 * grid_size.x,
                    y: tilemap_size.y as f32 * grid_size.y * 0.75,
                },
                TilemapType::Isometric(IsoCoordSystem::Diamond) => {
                    let topleft = iso_projection(Vec2::ZERO, &tilemap_size, &grid_size);
                    let topright = iso_projection(
                        Vec2 {
                            x: tilemap_size.x as f32 * grid_size.y,
                            y: 0.,
                        },
                        &tilemap_size,
                        &grid_size,
                    );

                    2. * (topright - topleft)
                }
                TilemapType::Isometric(IsoCoordSystem::Staggered) => {
                    panic!("Isometric (Staggered) map is not supported");
                }
                _ => unreachable!(),
            },
        };

        #[cfg(feature = "user_properties")]
        let properties =
            DeserializedMapProperties::load(&map, self.registry.read().deref(), load_context);

        #[cfg(feature = "user_properties")]
        trace!(?properties, "user properties");
        trace!(?tilesets, "tilesets");

        let asset_map = TiledMap {
            map,
            tilemap_size,
            tiled_offset,
            rect,
            topleft_chunk: topleft,
            bottomright_chunk: bottomright,
            tilesets,
            #[cfg(feature = "user_properties")]
            properties,
        };
        debug!(
            "Loaded map '{}': {:?}",
            load_context.path().display(),
            &asset_map,
        );
        Ok(asset_map)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}
