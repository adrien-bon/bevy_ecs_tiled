use bevy::{asset::{io::Reader, AssetLoader, LoadContext}, prelude::*};


/// Tiled world `Asset`.
///
/// `Asset` holding Tiled world informations.
#[derive(TypePath, Asset)]
pub struct TiledWorld {
    pub maps: Vec<(String, Vec2)>,
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
        // XXX: TODO
        Ok(TiledWorld {
            maps: vec!(
                ("finite.tmx".to_string(), Vec2::default()),
            ),
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["world"];
        EXTENSIONS
    }
}
