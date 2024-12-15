use std::{io::{Cursor, Error as IoError, ErrorKind, Read}, path::Path, sync::Arc};
use bevy::{asset::{io::Reader, AssetLoader, LoadContext}, prelude::*};


/// Tiled world `Asset`.
///
/// `Asset` holding Tiled world informations.
#[derive(TypePath, Asset)]
pub struct TiledWorld {
    pub maps: Vec<(String, Rect)>,
}

/// [TiledWorldMap] loading error.
#[derive(Debug, thiserror::Error)]
pub enum TiledWorldLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Default)]
pub(crate) struct TiledWorldLoader;

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
        let world = {
            let mut loader = tiled::Loader::with_cache_and_reader(
                tiled::DefaultResourceCache::new(),
                BytesResourceReader::new(&bytes, load_context),
            );
            loader.load_world(&world_path).map_err(|e| {
                std::io::Error::new(ErrorKind::Other, format!("Could not load Tiled world: {e}"))
            })?
        };

        let mut maps = Vec::new();

        for map in world.maps.unwrap().iter() {
            maps.push((
                map.filename.clone(),
                Rect::new(
                    map.x as f32, 
                    -map.y as f32,  // Invert for Tiled to Bevy Y axis
                    map.x as f32 + map.width.unwrap() as f32, 
                    -map.y as f32 + map.height.unwrap() as f32
                ),
            ));
        }

        Ok(TiledWorld { maps })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["world"];
        EXTENSIONS
    }
}
