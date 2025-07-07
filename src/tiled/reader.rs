//! Implementation of a custom [tiled::ResourceReader] for asset loading in Bevy.
//!
//! This module provides an implementation of the [`tiled::ResourceReader`] trait,
//! allowing Tiled assets (such as maps and tilesets) to be loaded from Bevy's asset system. This enables
//! seamless integration of Tiled resources with Bevy's asynchronous asset loading pipeline.
//!
//! The reader supports loading external tileset files (`.tsx`) as well as embedded resources from memory.

use bevy::asset::LoadContext;
use std::{
    io::{Cursor, Error as IoError, ErrorKind, Read},
    path::Path,
    sync::Arc,
};

/// A [`tiled::ResourceReader`] implementation for reading Tiled resources from Bevy's asset system.
///
/// This reader allows Tiled to load both embedded resources and external files (such as `.tsx` tilesets)
/// using Bevy's [`LoadContext`]. It supports asynchronous asset loading and provides the required interface
/// for the Tiled crate to access map and tileset data.
pub(crate) struct BytesResourceReader<'a, 'b> {
    /// The bytes of the main resource (e.g., the Tiled map file).
    bytes: Arc<[u8]>,
    /// The Bevy asset loading context.
    context: &'a mut LoadContext<'b>,
}

impl<'a, 'b> BytesResourceReader<'a, 'b> {
    /// Creates a new [`BytesResourceReader`] from the given bytes and asset loading context.
    pub(crate) fn new(bytes: &'a [u8], context: &'a mut LoadContext<'b>) -> Self {
        Self {
            bytes: Arc::from(bytes),
            context,
        }
    }
}

impl<'a> tiled::ResourceReader for BytesResourceReader<'a, '_> {
    type Resource = Box<dyn Read + 'a>;
    type Error = IoError;

    /// Reads a resource from the given path.
    ///
    /// If the path has a `.tsx` extension, the reader attempts to load the external tileset file
    /// using Bevy's asset system. Otherwise, it returns the embedded bytes.
    fn read_from(&mut self, path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        if let Some(extension) = path.extension() {
            if extension == "tsx" || extension == "tx" {
                let future = self.context.read_asset_bytes(path.to_path_buf());
                let data = futures_lite::future::block_on(future)
                    .map_err(|err| IoError::new(ErrorKind::NotFound, err))?;
                return Ok(Box::new(Cursor::new(data)));
            }
        }
        Ok(Box::new(Cursor::new(self.bytes.clone())))
    }
}
