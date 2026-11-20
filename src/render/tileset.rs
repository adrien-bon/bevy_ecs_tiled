//! Textures of Tiled tilesets and image layers.
//!
//! They are loaded along with the [`TiledMapAsset`], with the `render` feature only.

use crate::prelude::*;
use bevy::{
    asset::{AssetPath, LoadContext},
    platform::collections::HashMap,
    prelude::*,
};

/// Textures of a tileset, to render its tiles.
#[derive(Debug)]
pub(crate) struct TiledTilesetTextures {
    /// Whether this tileset can be used for tiles layers.
    ///
    /// A tileset can be used for tiles layers only if all the images it contains have the
    /// same dimensions (restriction from `bevy_ecs_tilemap`).
    pub(crate) usable_for_tiles_layer: bool,
    /// Tileset texture (ie. a single image or an images collection)
    pub(crate) tilemap_texture: TilemapTexture,
    /// The [`TextureAtlasLayout`] of a single image tileset, if any.
    pub(crate) texture_atlas_layout_handle: Option<Handle<TextureAtlasLayout>>,
    /// The index in the images collection of each tile.
    #[cfg(not(feature = "atlas"))]
    pub(crate) tile_image_offsets: HashMap<tiled::TileId, u32>,
}

/// Load the textures of a tileset, or `None` if they cannot be used.
pub(crate) fn load_tileset_textures(
    tileset: &tiled::Tileset,
    load_context: &mut LoadContext<'_>,
    path: &str,
) -> Option<TiledTilesetTextures> {
    #[cfg(not(feature = "atlas"))]
    let tileset_path = tileset.source.to_str()?;

    let mut texture_atlas_layout_handle = None;
    #[cfg(not(feature = "atlas"))]
    let mut tile_image_offsets = HashMap::default();
    let (usable_for_tiles_layer, tilemap_texture) = match &tileset.image {
        None => {
            // Already skipped by the map loader
            #[cfg(feature = "atlas")]
            {
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
                        trace!(
                            "Loading tile image from {asset_path:?} as image ({tileset_path}, {tile_id})"
                        );
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
            let texture: Handle<Image> = load_context.load(asset_path);

            let columns = tileset.columns;
            if columns > 0 {
                texture_atlas_layout_handle = load_context
                    .labeled_asset_scope(path.to_owned(), |_| {
                        Ok::<_, ()>(TextureAtlasLayout::from_grid(
                            UVec2::new(tileset.tile_width, tileset.tile_height),
                            columns,
                            tileset.tilecount / columns,
                            Some(UVec2::splat(tileset.spacing)),
                            Some(UVec2::splat(tileset.margin)),
                        ))
                    })
                    .ok();
            }

            (true, TilemapTexture::Single(texture.clone()))
        }
    };

    Some(TiledTilesetTextures {
        usable_for_tiles_layer,
        tilemap_texture,
        texture_atlas_layout_handle,
        #[cfg(not(feature = "atlas"))]
        tile_image_offsets,
    })
}

/// Load image assets used in image layers
pub(crate) fn load_image_layers<'a>(
    layers: impl Iterator<Item = tiled::Layer<'a>>,
    images: &mut HashMap<LayerId, Handle<Image>>,
    load_context: &mut LoadContext,
) {
    for layer in layers {
        match layer.layer_type() {
            tiled::LayerType::Image(image_layer) => {
                let Some(image) = &image_layer.image else {
                    continue;
                };

                let asset_path = AssetPath::from(image.source.clone());
                let handle: Handle<Image> = load_context.load(asset_path);

                images.insert(LayerId(layer.id()), handle);
            }
            tiled::LayerType::Group(group_layer) => {
                load_image_layers(group_layer.layers(), images, load_context);
            }
            _ => {}
        }
    }
}
