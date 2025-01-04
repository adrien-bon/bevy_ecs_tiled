//! This module contains all [Asset]s definition.

use std::io::{Cursor, Error as IoError, ErrorKind, Read};
#[cfg(feature = "user_properties")]
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "user_properties")]
use bevy::reflect::TypeRegistryArc;

#[cfg(feature = "user_properties")]
use crate::properties::load::DeserializedMapProperties;

use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, LoadContext, LoadedAsset},
    prelude::*,
    utils::HashMap,
};

use bevy_ecs_tilemap::prelude::*;

/// Tiled map `Asset`.
///
/// `Asset` holding Tiled map informations.
#[derive(TypePath, Asset)]
pub struct TiledMap {
    /// The raw Tiled map
    pub map: tiled::Map,

    /// HashMap of the map tilesets.
    ///
    /// Key is the Tiled tileset index.
    /// Value is a tuple of a boolean saying if this tileset can be used for tiles layer
    /// and the tileset texture (ie. a single image or a collection of images)
    /// A tileset can be used for tiles layer only if all the images it contains have the
    /// same dimensions (restriction from bevy_ecs_tilemap).
    pub tilemap_textures: HashMap<usize, (bool, TilemapTexture)>,

    #[cfg(feature = "user_properties")]
    pub(crate) properties: DeserializedMapProperties,

    /// The offset into the tileset_images for each tile id within each tileset.
    #[cfg(not(feature = "atlas"))]
    pub tile_image_offsets: HashMap<(usize, tiled::TileId), u32>,

    /// The [TextureAtlasLayout] handles associated to each tileset.
    pub texture_atlas_layout: HashMap<usize, Handle<TextureAtlasLayout>>,
}

struct BytesResourceReader<'a, 'b> {
    bytes: Arc<[u8]>,
    context: &'a mut LoadContext<'b>,
}
impl<'a, 'b> BytesResourceReader<'a, 'b> {
    fn new(bytes: &'a [u8], context: &'a mut LoadContext<'b>) -> Self {
        Self {
            bytes: Arc::from(bytes),
            context,
        }
    }
}

impl<'a> tiled::ResourceReader for BytesResourceReader<'a, '_> {
    type Resource = Box<dyn Read + 'a>;
    type Error = IoError;

    fn read_from(&mut self, path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        if let Some(extension) = path.extension() {
            if extension == "tsx" {
                let future = self.context.read_asset_bytes(path.to_path_buf());
                let data = futures_lite::future::block_on(future)
                    .map_err(|err| IoError::new(ErrorKind::NotFound, err))?;
                return Ok(Box::new(Cursor::new(data)));
            }
        }
        Ok(Box::new(Cursor::new(self.bytes.clone())))
    }
}

pub(crate) struct TiledLoader {
    #[cfg(feature = "user_properties")]
    pub registry: TypeRegistryArc,
}

impl FromWorld for TiledLoader {
    fn from_world(_world: &mut World) -> Self {
        Self {
            #[cfg(feature = "user_properties")]
            registry: _world.resource::<AppTypeRegistry>().0.clone(),
        }
    }
}

/// [TiledMap] loading error.
#[derive(Debug, thiserror::Error)]
pub enum TiledAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for TiledLoader {
    type Asset = TiledMap;
    type Settings = ();
    type Error = TiledAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        log::info!("Start loading map '{}'", load_context.path().display());

        let map_path = load_context.path().to_path_buf();
        let map = {
            // Allow the loader to also load tileset images.
            let mut loader = tiled::Loader::with_cache_and_reader(
                tiled::DefaultResourceCache::new(),
                BytesResourceReader::new(&bytes, load_context),
            );
            // Load the map and all tiles.
            loader.load_tmx_map(&map_path).map_err(|e| {
                std::io::Error::new(ErrorKind::Other, format!("Could not load TMX map: {e}"))
            })?
        };

        let mut tilemap_textures = HashMap::default();
        let mut texture_atlas_layout = HashMap::default();
        #[cfg(not(feature = "atlas"))]
        let mut tile_image_offsets = HashMap::default();

        for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
            let (usable_for_tiles_layer, tilemap_texture) = match &tileset.image {
                None => {
                    #[cfg(feature = "atlas")]
                    {
                        log::info!("Skipping image collection tileset '{}' which is incompatible with atlas feature", tileset.name);
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
                                log::debug!("Loading tile image from {asset_path:?} as image ({tileset_index}, {tile_id})");
                                let texture: Handle<Image> = load_context.load(asset_path.clone());
                                tile_image_offsets
                                    .insert((tileset_index, tile_id), tile_images.len() as u32);
                                tile_images.push(texture.clone());
                                if let Some(image_size) = image_size {
                                    if img.width != image_size.0 || img.height != image_size.1 {
                                        usable_for_tiles_layer = false;
                                    }
                                } else {
                                    image_size = Some((img.width, img.height));
                                }
                            }
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
                        let layout = TextureAtlasLayout::from_grid(
                            UVec2::new(tileset.tile_width, tileset.tile_height),
                            columns,
                            tileset.tilecount / columns,
                            Some(UVec2::new(tileset.spacing, tileset.spacing)),
                            Some(UVec2::new(
                                tileset.offset_x as u32 + tileset.margin,
                                tileset.offset_y as u32 + tileset.margin,
                            )),
                        );
                        let atlas_handle = load_context.add_loaded_labeled_asset(
                            tileset.name.clone(),
                            LoadedAsset::from(layout),
                        );
                        texture_atlas_layout.insert(tileset_index, atlas_handle);
                    }

                    (true, TilemapTexture::Single(texture.clone()))
                }
            };

            if !usable_for_tiles_layer {
                log::warn!(
                    "Tileset (index={:?}) cannot be used for tiles layer",
                    tileset_index
                );
            }
            tilemap_textures.insert(tileset_index, (usable_for_tiles_layer, tilemap_texture));
        }

        #[cfg(feature = "user_properties")]
        let properties =
            DeserializedMapProperties::load(&map, self.registry.read().deref(), load_context);

        #[cfg(feature = "user_properties")]
        trace!(?properties, "user properties");

        let asset_map = TiledMap {
            map,
            tilemap_textures,
            #[cfg(feature = "user_properties")]
            properties,
            #[cfg(not(feature = "atlas"))]
            tile_image_offsets,
            texture_atlas_layout,
        };

        log::info!("Loaded map '{}'", load_context.path().display());
        Ok(asset_map)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}
