//! This module contains all world [Asset]s definition.

use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, LoadContext},
    prelude::*,
};
use std::{fmt, io::ErrorKind};

use crate::{cache::TiledResourceCache, reader::BytesResourceReader, TiledMap};

use super::TiledMapAnchor;

/// Tiled world `Asset`.
///
/// `Asset` holding Tiled world informations.
#[derive(TypePath, Asset)]
pub struct TiledWorld {
    /// The raw Tiled world data
    pub world: tiled::World,
    /// World bounding box, unanchored
    ///
    /// Minimum is set at `(0., 0.)`
    /// Maximum is set at `(world_size.x, world_size.y)`
    pub rect: Rect,
    /// List of all the maps contained in this world
    ///
    /// Contains both the [TiledMap] handle and its associated [Rect] boundary
    /// as defined by the `.world` file.
    /// Note that the actual map boundaries are not taken into account for world chunking.
    pub maps: Vec<(Rect, Handle<TiledMap>)>,
}

impl TiledWorld {
    /// Offset that should be applied to world underlying maps to account for the [TiledMapAnchor]
    pub(crate) fn offset(&self, anchor: &TiledMapAnchor) -> Vec3 {
        match anchor {
            TiledMapAnchor::Center => Vec3 {
                x: -self.rect.width() / 2.0,
                y: -self.rect.height() / 2.0,
                z: 0.0,
            },
            TiledMapAnchor::BottomLeft => Vec3::ZERO,
        }
    }
}

impl fmt::Debug for TiledWorld {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TiledWorld")
            .field("world.source", &self.world.source)
            .field("rect", &self.rect)
            .finish()
    }
}

/// [TiledWorld] loading error.
#[derive(Debug, thiserror::Error)]
pub enum TiledWorldLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
    #[error("No map found in this world")]
    EmptyWorld,
    #[error("Infinite map found in this world (not supported)")]
    WorldWithInfiniteMap,
}

pub(crate) struct TiledWorldLoader {
    cache: TiledResourceCache,
}

impl FromWorld for TiledWorldLoader {
    fn from_world(world: &mut World) -> Self {
        Self {
            cache: world.resource::<TiledResourceCache>().clone(),
        }
    }
}

impl AssetLoader for TiledWorldLoader {
    type Asset = TiledWorld;
    type Settings = ();
    type Error = TiledWorldLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        debug!("Start loading world '{}'", load_context.path().display());

        let world_path = load_context.path().to_path_buf();

        let world = {
            let mut loader = tiled::Loader::with_cache_and_reader(
                self.cache.clone(),
                BytesResourceReader::new(&bytes, load_context),
            );
            loader.load_world(&world_path).map_err(|e| {
                std::io::Error::new(ErrorKind::Other, format!("Could not load Tiled world: {e}"))
            })?
        };

        if world.maps.is_empty() {
            return Err(TiledWorldLoaderError::EmptyWorld);
        }

        // Calculate the full rect of the world
        let mut world_rect = Rect::new(0.0, 0.0, 0.0, 0.0);
        for map in world.maps.iter() {
            let (Some(map_width), Some(map_height)) = (map.width, map.height) else {
                // Assume that we cannot get map width / map height because it's an infinite map
                return Err(TiledWorldLoaderError::WorldWithInfiniteMap);
            };
            let map_rect = Rect::new(
                map.x as f32,
                map.y as f32, // Invert for Tiled to Bevy Y axis
                map.x as f32 + map_width as f32,
                map.y as f32 + map_height as f32,
            );

            world_rect = world_rect.union(map_rect);
        }

        // Load all maps
        let mut maps = Vec::new();
        for map in world.maps.iter() {
            // Seems safe to unwrap() here since we do it on the world path (which should always have a parent)
            let map_path = world_path.parent().unwrap().join(map.filename.clone());

            let (Some(map_width), Some(map_height)) = (map.width, map.height) else {
                // Assume that we cannot get map width / map height because it's an infinite map
                return Err(TiledWorldLoaderError::WorldWithInfiniteMap);
            };

            maps.push((
                Rect::new(
                    map.x as f32,
                    world_rect.max.y - map_height as f32 - map.y as f32, // Invert for Tiled to Bevy Y axis
                    map.x as f32 + map_width as f32,
                    world_rect.max.y - map.y as f32,
                ),
                load_context.load(AssetPath::from(map_path)),
            ));
        }

        trace!(?maps, "maps");

        let world = TiledWorld {
            world,
            rect: world_rect,
            maps,
        };
        debug!(
            "Loaded world '{}': {:?}",
            load_context.path().display(),
            world
        );
        Ok(world)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["world"];
        EXTENSIONS
    }
}
