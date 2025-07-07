//! Resource cache implementation for Tiled assets.
//!
//! This module provides a thread-safe wrapper around [`tiled::DefaultResourceCache`].
//! It implements the [`tiled::ResourceCache`] trait, enabling efficient caching and retrieval of Tiled tilesets
//! and templates within the Bevy ECS environment. The cache is stored as a Bevy resource and is accessible
//! throughout the application for asset loading and management.
//!
//! The cache supports concurrent access and can be cleared at runtime if needed.

use bevy::prelude::*;
use std::sync::{Arc, RwLock};
use tiled::{DefaultResourceCache, ResourceCache};

/// Thread-safe resource cache for Tiled assets, stored as a Bevy resource.
///
/// Wraps a [`tiled::DefaultResourceCache`] in an [`Arc<RwLock<...>>`] to allow safe concurrent access
/// from multiple systems. Provides methods for clearing the cache and implements the [`tiled::ResourceCache`] trait.
#[derive(Resource, Clone)]
pub(crate) struct TiledResourceCache(pub(crate) Arc<RwLock<DefaultResourceCache>>);

impl TiledResourceCache {
    /// Creates a new, empty Tiled resource cache.
    pub(crate) fn new() -> Self {
        Self(Arc::new(RwLock::new(DefaultResourceCache::new())))
    }
}

impl TiledResourceCache {
    /// Clears all cached tilesets and templates.
    ///
    /// This can be useful to force reloading of Tiled assets at runtime.
    pub fn clear(&mut self) {
        debug!("Clearing cache");
        *self.0.write().unwrap() = DefaultResourceCache::new();
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
        tileset: Arc<tiled::Tileset>,
    ) {
        self.0.write().unwrap().insert_tileset(path, tileset);
    }

    fn insert_template(
        &mut self,
        path: impl AsRef<tiled::ResourcePath>,
        template: Arc<tiled::Template>,
    ) {
        self.0.write().unwrap().insert_template(path, template);
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(TiledResourceCache::new());
}
