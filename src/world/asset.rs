use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, LoadContext},
    prelude::*,
};
use std::io::ErrorKind;

use crate::{TiledMap, cache::TiledResourceCache, reader::BytesResourceReader};

/// Tiled world `Asset`.
///
/// `Asset` holding Tiled world informations.
#[derive(TypePath, Asset)]
pub struct TiledWorld {
    pub world: tiled::World,

    pub world_rect: Rect,

    pub maps: Vec<(Rect, Handle<TiledMap>)>,
}

/// [TiledWorldMap] loading error.
#[derive(Debug, thiserror::Error)]
pub enum TiledWorldLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
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

        log::info!("Start loading world '{}'", load_context.path().display());

        let world_path = load_context.path().to_path_buf();

        let mut world = {
            let mut loader = tiled::Loader::with_cache_and_reader(
                self.cache.clone(),
                BytesResourceReader::new(&bytes, load_context),
            );
            loader.load_world(&world_path).map_err(|e| {
                std::io::Error::new(ErrorKind::Other, format!("Could not load Tiled world: {e}"))
            })?
        };

        // Calculate the full rect of the world
        let mut world_rect = Rect::new(0.0, 0.0, 0.0, 0.0);

        for map in world.maps.as_ref().unwrap().iter() {
            let map_rect = Rect::new(
                map.x as f32,
                map.y as f32, // Invert for Tiled to Bevy Y axis
                map.x as f32 + map.width.unwrap() as f32,
                map.y as f32 + map.height.unwrap() as f32,
            );

            world_rect = world_rect.union(map_rect);
        }

        // Load all maps
        let mut maps = Vec::new();

        for map in world.maps.take().unwrap().iter() {
            let asset_path =
                AssetPath::from(world_path.parent().unwrap().join(map.filename.clone()));

            let map_handle: Handle<TiledMap> = load_context.load(asset_path);

            let map_height = map.height.unwrap() as f32;

            // Position maps
            maps.push((
                Rect::new(
                    map.x as f32,
                    world_rect.max.y - map_height - map.y as f32, // Invert for Tiled to Bevy Y axis
                    map.x as f32 + map.width.unwrap() as f32,
                    world_rect.max.y - map.y as f32,
                ),
                map_handle,
            ));
        }

        Ok(TiledWorld {
            world,
            world_rect,
            maps,
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["world"];
        EXTENSIONS
    }
}
