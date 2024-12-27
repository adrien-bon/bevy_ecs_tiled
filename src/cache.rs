use std::sync::{Arc, RwLock};
use bevy::prelude::*;
use tiled::{DefaultResourceCache, ResourceCache};

#[derive(Resource, Clone)]
pub(crate) struct TiledResourceCache(pub(crate) Arc<RwLock<DefaultResourceCache>>);

impl TiledResourceCache {
    pub(crate) fn new() -> Self {
        Self(
            Arc::new(RwLock::new(DefaultResourceCache::new()))
        )
    }
}

impl ResourceCache for TiledResourceCache {
    fn get_tileset(
        &self,
        path: impl AsRef<tiled::ResourcePath>,
    ) -> Option<std::sync::Arc<tiled::Tileset>> {
        self.0.read().unwrap().get_tileset(path)
    }

    fn get_template(
        &self,
        path: impl AsRef<tiled::ResourcePath>,
    ) -> Option<std::sync::Arc<tiled::Template>> {
        self.0.read().unwrap().get_template(path)
    }

    fn insert_tileset(
        &mut self,
        path: impl AsRef<tiled::ResourcePath>,
        tileset: Arc<tiled::Tileset>
    ) {
        self.0.write().unwrap().insert_tileset(path, tileset);
    }

    fn insert_template(
        &mut self,
        path: impl AsRef<tiled::ResourcePath>,
        template: Arc<tiled::Template>
    ) {
        self.0.write().unwrap().insert_template(path, template);
    }
}
