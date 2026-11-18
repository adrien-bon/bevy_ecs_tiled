//! Resource cache implementation for Tiled assets.
//!
//! This module provides a thread-safe wrapper around [`tiled::DefaultResourceCache`].
//! It implements the [`tiled::ResourceCache`] trait, enabling efficient caching and retrieval of Tiled tilesets
//! and templates within the Bevy ECS environment. The cache is stored as a Bevy resource and is accessible
//! throughout the application for asset loading and management.
//!
//! The cache supports concurrent access and can be cleared at runtime if needed.

use bevy::prelude::*;
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Thread-safe resource cache for Tiled assets, stored as a Bevy resource.
///
/// Wraps a [`tiled::DefaultResourceCache`] in an [`Arc<RwLock<...>>`] to allow safe concurrent access
/// from multiple systems. Provides methods for clearing the cache and implements the [`tiled::ResourceCache`] trait.
#[derive(Resource, Clone, Default)]
pub(crate) struct TiledResourceCache(pub(crate) Arc<RwLock<tiled::DefaultResourceCache>>);

impl TiledResourceCache {
    // A panic while holding the lock cannot leave the cache in an inconsistent state:
    // ignore lock poisoning instead of propagating the panic to every asset loader
    fn read(&self) -> RwLockReadGuard<'_, tiled::DefaultResourceCache> {
        self.0.read().unwrap_or_else(PoisonError::into_inner)
    }

    fn write(&self) -> RwLockWriteGuard<'_, tiled::DefaultResourceCache> {
        self.0.write().unwrap_or_else(PoisonError::into_inner)
    }

    /// Clears all cached tilesets and templates.
    ///
    /// This can be useful to force reloading of Tiled assets at runtime.
    pub fn clear(&mut self) {
        debug!("Clearing cache");
        *self.write() = tiled::DefaultResourceCache::new();
    }
}

impl tiled::ResourceCache for TiledResourceCache {
    fn get_tileset(
        &self,
        path: impl AsRef<tiled::ResourcePath>,
    ) -> Option<std::sync::Arc<tiled::Tileset>> {
        self.read().get_tileset(path)
    }

    fn get_template(
        &self,
        path: impl AsRef<tiled::ResourcePath>,
    ) -> Option<std::sync::Arc<tiled::Template>> {
        self.read().get_template(path)
    }

    fn insert_tileset(
        &mut self,
        path: impl AsRef<tiled::ResourcePath>,
        tileset: Arc<tiled::Tileset>,
    ) {
        self.write().insert_tileset(path, tileset);
    }

    fn insert_template(
        &mut self,
        path: impl AsRef<tiled::ResourcePath>,
        template: Arc<tiled::Template>,
    ) {
        self.write().insert_template(path, template);
    }
}

pub(crate) fn plugin(app: &mut App) {
    // Asset loaders keep a clone of this cache: it must not be replaced once they are initialized
    app.init_resource::<TiledResourceCache>();
}
