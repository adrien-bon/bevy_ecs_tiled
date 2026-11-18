//! Asset loader for Tiled maps.
//!
//! This module defines the asset loader implementation for importing Tiled maps into Bevy's asset system.

#[cfg(feature = "user_properties")]
use std::ops::Deref;
use std::sync::Arc;

use crate::{
    asset::{
        cache::TiledResourceCache, helpers::hex_side_length, map::TiledMapTileset,
        reader::BytesResourceReader,
    },
    prelude::*,
};
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    prelude::*,
};

#[derive(TypePath)]
struct TiledMapLoader {
    cache: TiledResourceCache,
    #[cfg(feature = "user_properties")]
    registry: bevy::reflect::TypeRegistryArc,
}

pub(crate) fn tileset_label(tileset: &tiled::Tileset) -> Option<String> {
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
#[non_exhaustive]
pub enum TiledMapLoaderError {
    /// An [`IO`](std::io) Error
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
    /// The Tiled map (or one of its dependencies) could not be parsed
    #[error("Could not load TMX map: {0}")]
    Tiled(#[from] tiled::Error),
    /// The map orientation is not supported by this plugin
    #[error("Unsupported map orientation: {0:?}")]
    UnsupportedOrientation(tiled::Orientation),
}

fn process_layers<'a>(
    layers: impl Iterator<Item = tiled::Layer<'a>>,
    largest_tile_size: &mut TilemapTileSize,
    tilesets: &mut HashMap<TilesetId, TiledMapTileset>,
    tilesets_id_by_label: &mut HashMap<String, TilesetId>,
    load_context: &mut LoadContext,
) {
    for layer in layers {
        match layer.layer_type() {
            tiled::LayerType::Tiles(tiles_layer) => {
                // Iterate over tiles layers to find the largest tile_size of the map
                match tiles_layer {
                    tiled::TileLayer::Finite(tiles_finite_layer) => {
                        for x in 0..tiles_finite_layer.width() as i32 {
                            for y in 0..tiles_finite_layer.height() as i32 {
                                let Some(tile) =
                                    tiles_finite_layer.get_tile(x, y).and_then(|t| t.get_tile())
                                else {
                                    continue;
                                };
                                let tile_size_val = crate::asset::helpers::tile_size(&tile);
                                if tile_size_val > *largest_tile_size {
                                    debug!("Update tile_size: {tile_size_val:?}");
                                    *largest_tile_size = tile_size_val;
                                }
                            }
                        }
                    }
                    tiled::TileLayer::Infinite(tiles_infinite_layer) => {
                        for (_, chunk) in tiles_infinite_layer.chunks() {
                            for x in 0..tiled::ChunkData::WIDTH as i32 {
                                for y in 0..tiled::ChunkData::HEIGHT as i32 {
                                    let Some(tile) =
                                        chunk.get_tile(x, y).and_then(|t| t.get_tile())
                                    else {
                                        continue;
                                    };
                                    let tile_size_val = crate::asset::helpers::tile_size(&tile);
                                    if tile_size_val > *largest_tile_size {
                                        debug!("Update tile_size: {tile_size_val:?}");
                                        *largest_tile_size = tile_size_val;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            tiled::LayerType::Objects(object_layer) => {
                // Iterate over objects layers to load potential external tilesets (templates)
                for object_data in object_layer.objects() {
                    let Some(tile) = object_data.get_tile() else {
                        continue;
                    };

                    let tiled::TilesetLocation::Template(tileset) = tile.tileset_location() else {
                        continue;
                    };

                    let Some(label) = tileset_label(tileset) else {
                        continue;
                    };

                    if tilesets_id_by_label.contains_key(&label) {
                        continue;
                    }

                    let Some(tiled_map_tileset) =
                        tileset_to_tiled_map_tileset(tileset.clone(), load_context, &label)
                    else {
                        continue;
                    };

                    // Template tilesets get an ID after all the map tilesets
                    let id = next_tileset_id(tilesets);
                    tilesets_id_by_label.insert(label, id);
                    tilesets.insert(id, tiled_map_tileset);
                }
            }
            tiled::LayerType::Image(_) => {}
            tiled::LayerType::Group(group_layer) => {
                process_layers(
                    group_layer.layers(),
                    largest_tile_size,
                    tilesets,
                    tilesets_id_by_label,
                    load_context,
                );
            }
        }
    }
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

        debug!("Start loading map '{}'", load_context.path());

        // Pre-load all external tilesets/templates for WASM compatibility.
        // This is necessary because we cannot block on async operations in WASM.
        let external_cache =
            crate::asset::reader::preload_external_resources(&bytes, load_context).await;

        let map_path = load_context.path().path().to_path_buf();
        let map = {
            // Allow the loader to also load tileset images.
            let mut loader = tiled::Loader::with_cache_and_reader(
                self.cache.clone(),
                BytesResourceReader::new(&bytes, &external_cache),
            );
            // Load the map and all tiles.
            loader.load_tmx_map(map_path)?
        };

        let grid_size = crate::asset::helpers::grid_size_from_map(&map);
        let mut tilesets = HashMap::default();
        let mut tilesets_id_by_label = HashMap::default();

        for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
            debug!(
                "Loading tileset (index={:?} name={:?}) from {:?}",
                tileset_index, tileset.name, tileset.source
            );

            let Some(label) = tileset_label(tileset) else {
                continue;
            };

            let Some(tiled_map_tileset) =
                tileset_to_tiled_map_tileset(tileset.clone(), load_context, &label)
            else {
                continue;
            };

            tilesets_id_by_label.insert(label, TilesetId(tileset_index as u32));
            tilesets.insert(TilesetId(tileset_index as u32), tiled_map_tileset);
        }

        let mut largest_tile_size =
            TilemapTileSize::new(map.tile_width as f32, map.tile_height as f32);

        // Start processing from root layers
        process_layers(
            map.layers(),
            &mut largest_tile_size,
            &mut tilesets,
            &mut tilesets_id_by_label,
            load_context,
        );

        #[cfg(feature = "render")]
        let images = {
            let mut images = HashMap::default();
            crate::render::tileset::load_image_layers(map.layers(), &mut images, load_context);
            images
        };

        // Determine top left and bottom right chunk indexes of all infinite layers for this map
        let chunks_bounds = infinite_layers_chunks_bounds(map.layers());

        let (tilemap_size, topleft, bottomright) =
            if let Some((topleft, bottomright)) = chunks_bounds {
                debug!(
                    "(infinite map) topleft = {:?}, bottomright = {:?}",
                    topleft, bottomright
                );
                (
                    TilemapSize {
                        x: (bottomright.0 - topleft.0 + 1) as u32 * tiled::ChunkData::WIDTH,
                        y: (bottomright.1 - topleft.1 + 1) as u32 * tiled::ChunkData::HEIGHT,
                    },
                    topleft,
                    bottomright,
                )
            } else {
                (
                    TilemapSize {
                        x: map.width,
                        y: map.height,
                    },
                    (0, 0),
                    (0, 0),
                )
            };

        // Hexagonal maps coordinate system depends on the map size parity
        let Some(map_type) = crate::asset::helpers::tilemap_type_from_map(&map, &tilemap_size)
        else {
            return Err(TiledMapLoaderError::UnsupportedOrientation(map.orientation));
        };

        let tiled_offset = if chunks_bounds.is_some() {
            match map_type {
                TilemapType::Square => Vec2 {
                    x: -topleft.0 as f32 * tiled::ChunkData::WIDTH as f32 * grid_size.x,
                    y: topleft.1 as f32 * tiled::ChunkData::HEIGHT as f32 * grid_size.y,
                },
                TilemapType::Hexagon(coord_system) if is_column_hex(coord_system) => Vec2 {
                    x: -topleft.0 as f32 * tiled::ChunkData::WIDTH as f32 * grid_size.x * 0.75,
                    y: topleft.1 as f32 * tiled::ChunkData::HEIGHT as f32 * grid_size.y,
                },
                TilemapType::Hexagon(_) => Vec2 {
                    x: -topleft.0 as f32 * tiled::ChunkData::WIDTH as f32 * grid_size.x,
                    y: topleft.1 as f32 * tiled::ChunkData::HEIGHT as f32 * grid_size.y * 0.75,
                },
                TilemapType::Isometric(_) => Vec2 {
                    x: -topleft.0 as f32 * tiled::ChunkData::WIDTH as f32 * grid_size.y,
                    y: -topleft.1 as f32 * tiled::ChunkData::HEIGHT as f32 * grid_size.y,
                },
            }
        } else {
            Vec2::ZERO
        };

        let rect = Rect {
            min: Vec2::ZERO,
            max: match map_type {
                TilemapType::Square => Vec2 {
                    x: tilemap_size.x as f32 * grid_size.x,
                    y: tilemap_size.y as f32 * grid_size.y,
                },
                // For hexagonal maps, account for the last column (or row) hexagon tip
                // and for the half tile shift of staggered columns (or rows).
                // Staggered columns (or rows) are `0.75 * grid_size` apart (see `grid_size_from_map()`)
                TilemapType::Hexagon(coord_system) if is_column_hex(coord_system) => Vec2 {
                    x: tilemap_size.x as f32 * grid_size.x * 0.75
                        + (map.tile_width as f32 - hex_side_length(&map)) / 2.,
                    y: tilemap_size.y as f32 * grid_size.y
                        + if tilemap_size.x > 1 {
                            grid_size.y / 2.
                        } else {
                            0.
                        },
                },
                TilemapType::Hexagon(_) => Vec2 {
                    x: tilemap_size.x as f32 * grid_size.x
                        + if tilemap_size.y > 1 {
                            grid_size.x / 2.
                        } else {
                            0.
                        },
                    y: tilemap_size.y as f32 * grid_size.y * 0.75
                        + (map.tile_height as f32 - hex_side_length(&map)) / 2.,
                },
                // Diamond bounding box: each tile along X or Y axis adds half a tile
                // to both the map width and height
                TilemapType::Isometric(_) => Vec2 {
                    x: (tilemap_size.x + tilemap_size.y) as f32 * grid_size.x / 2.,
                    y: (tilemap_size.x + tilemap_size.y) as f32 * grid_size.y / 2.,
                },
            },
        };

        #[cfg(feature = "user_properties")]
        let properties = crate::properties::user::load::DeserializedMapProperties::load(
            &map,
            // Map tilesets, then tilesets only referenced by templates
            map.tilesets()
                .iter()
                .enumerate()
                .map(|(id, t)| (TilesetId(id as u32), t.as_ref()))
                .chain(
                    tilesets
                        .iter()
                        .filter(|(id, _)| id.0 as usize >= map.tilesets().len())
                        .map(|(id, t)| (*id, t.tileset.as_ref())),
                ),
            self.registry.read().deref(),
            load_context,
        );

        #[cfg(feature = "user_properties")]
        trace!(?properties, "user properties");
        trace!(?tilesets, "tilesets");

        let asset_map = TiledMapAsset {
            map,
            map_type,
            grid_size,
            tilemap_size,
            largest_tile_size,
            tiled_offset,
            rect,
            topleft_chunk: topleft,
            bottomright_chunk: bottomright,
            tilesets,
            tilesets_id_by_label,
            #[cfg(feature = "render")]
            images,
            #[cfg(feature = "user_properties")]
            properties,
        };
        debug!("Loaded map '{}': {:?}", load_context.path(), &asset_map,);
        Ok(asset_map)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

/// Build a [`TiledMapTileset`] from a Tiled tileset, or `None` if this tileset is skipped.
///
/// Which tilesets are skipped must not depend on the `render` feature:
/// it only controls whether their images are loaded.
#[cfg_attr(not(feature = "render"), allow(unused_variables))]
fn tileset_to_tiled_map_tileset(
    tileset: Arc<tiled::Tileset>,
    load_context: &mut LoadContext<'_>,
    path: &str,
) -> Option<TiledMapTileset> {
    #[cfg(feature = "atlas")]
    if tileset.image.is_none() {
        warn!(
            "Skipping image collection tileset '{}' which is incompatible with atlas feature",
            tileset.name
        );
        return None;
    }

    Some(TiledMapTileset {
        #[cfg(feature = "render")]
        textures: crate::render::tileset::load_tileset_textures(&tileset, load_context, path)?,
        tileset,
    })
}

/// Whether hexagons of this coordinate system are laid out in columns (ie. "flat-top" hexagons)
fn is_column_hex(coord_system: HexCoordSystem) -> bool {
    matches!(
        coord_system,
        HexCoordSystem::Column | HexCoordSystem::ColumnEven | HexCoordSystem::ColumnOdd
    )
}

/// Top-left and bottom-right chunk indexes among all the chunks of infinite tiles layers, if any.
///
/// The bottom-right chunk index is never lower than `(0, 0)`.
fn infinite_layers_chunks_bounds<'a>(
    layers: impl Iterator<Item = tiled::Layer<'a>>,
) -> Option<((i32, i32), (i32, i32))> {
    let mut bounds: Option<((i32, i32), (i32, i32))> = None;
    for layer in layers {
        let tiled::LayerType::Tiles(tiled::TileLayer::Infinite(layer)) = layer.layer_type() else {
            continue;
        };
        for (pos, _) in layer.chunks() {
            let (topleft, bottomright) = bounds.unwrap_or((pos, (0, 0)));
            bounds = Some((
                (topleft.0.min(pos.0), topleft.1.min(pos.1)),
                (bottomright.0.max(pos.0), bottomright.1.max(pos.1)),
            ));
        }
    }
    bounds
}

/// Returns the first tileset ID which is not used yet.
///
/// Map tilesets are inserted first using their index, so this ID never
/// conflicts with a map tileset, even if some of them failed to load.
fn next_tileset_id(tilesets: &HashMap<TilesetId, TiledMapTileset>) -> TilesetId {
    TilesetId(tilesets.keys().max().map_or(0, |id| id.0 + 1))
}

pub(crate) fn plugin(app: &mut App) {
    app.init_asset_loader::<TiledMapLoader>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::LoadState;
    use std::time::{Duration, Instant};

    #[test]
    fn unsupported_orientation_is_a_load_error() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_plugins((crate::asset::cache::plugin, crate::asset::map::plugin));

        let handle: Handle<TiledMapAsset> = app
            .world()
            .resource::<AssetServer>()
            .load("tests/loader/staggered_map.tmx");

        let deadline = Instant::now() + Duration::from_secs(10);
        let error = loop {
            assert!(Instant::now() < deadline, "test map did not load in time");
            app.update();
            match app.world().resource::<AssetServer>().load_state(&handle) {
                LoadState::Failed(error) => break error,
                LoadState::Loaded => panic!("staggered map should not load"),
                _ => std::thread::sleep(Duration::from_millis(1)),
            }
        };
        assert!(
            error.to_string().contains("Unsupported map orientation"),
            "unexpected error: {error}"
        );
    }
}
