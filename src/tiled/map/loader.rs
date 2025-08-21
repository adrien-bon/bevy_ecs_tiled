//! Asset loader for Tiled maps.
//!
//! This module defines the asset loader implementation for importing Tiled maps into Bevy's asset system.

#[cfg(feature = "user_properties")]
use std::ops::Deref;
use std::sync::Arc;

use crate::{
    prelude::*,
    tiled::{
        cache::TiledResourceCache, helpers::iso_projection, map::asset::TiledMapTileset,
        reader::BytesResourceReader,
    },
};
use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, LoadContext},
    prelude::*,
};
use bevy_ecs_tilemap::map::{HexCoordSystem, IsoCoordSystem, TilemapTexture};
use tiled::{ChunkData, LayerType, TilesetLocation};

struct TiledMapLoader {
    cache: TiledResourceCache,
    #[cfg(feature = "user_properties")]
    registry: bevy::reflect::TypeRegistryArc,
}

pub(crate) fn tileset_path(tileset: &Tileset) -> Option<String> {
    tileset
        .source
        .to_str()
        .map(|s| format!("{s}#{}", tileset.name))
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

/// [`TiledMapAsset`] loading error.
#[derive(Debug, thiserror::Error)]
pub enum TiledMapLoaderError {
    /// An [`IO`](std::io) Error
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for TiledMapLoader {
    type Asset = TiledMapAsset;
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
            loader
                .load_tmx_map(&map_path)
                .map_err(|e| std::io::Error::other(format!("Could not load TMX map: {e}")))?
        };

        let mut tilesets = HashMap::default();
        let mut tilesets_path_by_index = HashMap::<u32, String>::default();
        for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
            debug!(
                "Loading tileset (index={:?} name={:?}) from {:?}",
                tileset_index, tileset.name, tileset.source
            );

            let Some(path) = tileset_path(tileset) else {
                continue;
            };

            let Some(tiled_map_tileset) =
                tileset_to_tiled_map_tileset(tileset.clone(), load_context)
            else {
                continue;
            };

            tilesets_path_by_index.insert(tileset_index as u32, path.to_owned());
            tilesets.insert(path.to_owned(), tiled_map_tileset);
        }

        for layer in map.layers() {
            let LayerType::Objects(object_layer) = layer.layer_type() else {
                continue;
            };

            for object_data in object_layer.objects() {
                let Some(tile) = object_data.get_tile() else {
                    continue;
                };

                let TilesetLocation::Template(tileset) = tile.tileset_location() else {
                    continue;
                };

                let Some(path) = tileset_path(tileset) else {
                    continue;
                };

                if tilesets.contains_key(&path) {
                    continue;
                }

                let Some(tiled_map_tileset) =
                    tileset_to_tiled_map_tileset(tileset.clone(), load_context)
                else {
                    continue;
                };

                tilesets.insert(path.to_owned(), tiled_map_tileset);
            }
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

        let map_type = tilemap_type_from_map(&map);
        let grid_size = grid_size_from_map(&map);
        let tile_size = tile_size_from_map(&map);
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
                    let topleft = iso_projection(Vec2::ZERO, &tilemap_size, &tile_size);
                    let topright = iso_projection(
                        Vec2 {
                            x: tilemap_size.x as f32 * grid_size.y,
                            y: 0.,
                        },
                        &tilemap_size,
                        &tile_size,
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
        let properties = crate::tiled::properties::load::DeserializedMapProperties::load(
            &map,
            self.registry.read().deref(),
            load_context,
        );

        #[cfg(feature = "user_properties")]
        trace!(?properties, "user properties");
        trace!(?tilesets, "tilesets");

        let asset_map = TiledMapAsset {
            map,
            tilemap_size,
            tiled_offset,
            rect,
            topleft_chunk: topleft,
            bottomright_chunk: bottomright,
            tilesets,
            tilesets_path_by_index,
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

fn tileset_to_tiled_map_tileset(
    tileset: Arc<Tileset>,
    load_context: &mut LoadContext<'_>,
) -> Option<TiledMapTileset> {
    #[cfg(not(feature = "atlas"))]
    let tileset_path = tileset.source.to_str()?;

    let mut texture_atlas_layout_handle = None;
    #[cfg(not(feature = "atlas"))]
    let mut tile_image_offsets = HashMap::default();
    let (usable_for_tiles_layer, tilemap_texture) = match &tileset.image {
        None => {
            #[cfg(feature = "atlas")]
            {
                info!("Skipping image collection tileset '{}' which is incompatible with atlas feature", tileset.name);
                return None;
            }

            #[cfg(not(feature = "atlas"))]
            {
                let mut usable_for_tiles_layer = true;
                let mut image_size: Option<(i32, i32)> = None;
                let mut tile_images: Vec<Handle<Image>> = Vec::new();
                for (tile_id, tile) in tileset.tiles() {
                    if let Some(img) = &tile.image {
                        let asset_path = AssetPath::from(img.source.clone());
                        trace!("Loading tile image from {asset_path:?} as image ({tileset_path}, {tile_id})");
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
                        "Tileset (path={:?}) have non constant image size and cannot be used for tiles layer",
                        tileset_path
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
                texture_atlas_layout_handle =
                    Some(load_context.labeled_asset_scope(tileset.name.clone(), |_| {
                        TextureAtlasLayout::from_grid(
                            UVec2::new(tileset.tile_width, tileset.tile_height),
                            columns,
                            tileset.tilecount / columns,
                            Some(UVec2::splat(tileset.spacing)),
                            Some(UVec2::splat(tileset.margin)),
                        )
                    }));
            }

            (true, TilemapTexture::Single(texture.clone()))
        }
    };

    Some(TiledMapTileset {
        usable_for_tiles_layer,
        tilemap_texture,
        texture_atlas_layout_handle,
        #[cfg(not(feature = "atlas"))]
        tile_image_offsets,
    })
}

pub(crate) fn plugin(app: &mut App) {
    app.init_asset_loader::<TiledMapLoader>();
}
