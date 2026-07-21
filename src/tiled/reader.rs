//! Implementation of a custom [tiled::ResourceReader] for asset loading in Bevy.
//!
//! This module provides an implementation of the [`tiled::ResourceReader`] trait,
//! allowing Tiled assets (such as maps and tilesets) to be loaded from Bevy's asset system. This enables
//! seamless integration of Tiled resources with Bevy's asynchronous asset loading pipeline.
//!
//! The reader supports loading external tileset files (`.tsx`) as well as embedded resources from memory.

use bevy::asset::LoadContext;
use bevy::platform::collections::HashMap;
use std::{
    io::{Cursor, Error as IoError, ErrorKind, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

/// A [`tiled::ResourceReader`] implementation for reading Tiled resources from Bevy's asset system.
///
/// This reader allows Tiled to load both embedded resources and external files (such as `.tsx` tilesets)
/// using Bevy's [`LoadContext`]. It supports asynchronous asset loading and provides the required interface
/// for the Tiled crate to access map and tileset data.
pub(crate) struct BytesResourceReader<'a> {
    /// The bytes of the main resource (e.g., the Tiled map file).
    bytes: Arc<[u8]>,
    /// Pre-loaded external resources (tilesets, templates) for WASM compatibility.
    cache: &'a HashMap<PathBuf, Vec<u8>>,
}

impl<'a> BytesResourceReader<'a> {
    /// Creates a new [`BytesResourceReader`] from the given bytes and pre-loaded cache.
    pub(crate) fn new(bytes: &[u8], cache: &'a HashMap<PathBuf, Vec<u8>>) -> Self {
        Self {
            bytes: Arc::from(bytes),
            cache,
        }
    }
}

impl tiled::ResourceReader for BytesResourceReader<'_> {
    type Resource = Box<dyn Read>;
    type Error = IoError;

    /// Reads a resource from the given path.
    ///
    /// If the path has a `.tsx` or `.tx` extension, the reader looks up the pre-loaded cache.
    /// Otherwise, it returns the embedded bytes.
    fn read_from(&mut self, path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        if let Some(extension) = path.extension() {
            if extension == "tsx" || extension == "tx" {
                // Look up in pre-loaded cache
                // Normalize path to resolve `..` components (e.g. `maps/../tilesets/x.tsx` → `tilesets/x.tsx`)
                // so it matches the keys stored during pre-loading.
                let normalized = normalize_path(path);
                let data = self.cache.get(&normalized).ok_or_else(|| {
                    IoError::new(
                        ErrorKind::NotFound,
                        format!(
                            "External tileset/template '{}' not found in cache. For WASM builds, all external tilesets must be pre-loaded.",
                            path.display()
                        ),
                    )
                })?;
                return Ok(Box::new(Cursor::new(data.clone())));
            }
        }
        Ok(Box::new(Cursor::new(self.bytes.clone())))
    }
}

/// Normalizes a path by resolving `.` and `..` components in place,
/// e.g. `maps/../tilesets/collision.tsx` → `tilesets/collision.tsx`.
fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                result.pop();
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(s) => {
                result.push(s);
            }
            _ => {
                result.push(component);
            }
        }
    }
    result
}

/// Extract external tileset/template paths from TMX/TSX/TX XML content.
/// This does a simple regex-free parse to find source attributes.
pub(crate) fn extract_external_paths(xml_content: &[u8]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let content = String::from_utf8_lossy(xml_content);

    // Find tileset sources: <tileset ... source="path.tsx" ...>
    for line in content.lines() {
        if let Some(start) = line.find("source=\"") {
            let rest = &line[start + 8..];
            if let Some(end) = rest.find('"') {
                let path = &rest[..end];
                if path.ends_with(".tsx") || path.ends_with(".tx") {
                    paths.push(PathBuf::from(path));
                }
            }
        }
    }

    paths
}

/// Resolve a relative path against the asset's parent directory.
/// For example, if the asset is at `maps/01_first_street.tmx` and the
/// relative path is `../tilesets/kilowatt_tiles.tsx`, this returns
/// `tilesets/kilowatt_tiles.tsx`.
fn resolve_relative_path(asset_path: &Path, relative_path: &Path) -> PathBuf {
    if let Some(parent) = asset_path.parent() {
        // Join the parent directory with the relative path and normalize
        let joined = parent.join(relative_path);
        // Normalize the path by resolving .. and . components
        let mut components = Vec::new();
        for component in joined.components() {
            match component {
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::CurDir => {}
                c => components.push(c),
            }
        }
        components.iter().collect()
    } else {
        relative_path.to_path_buf()
    }
}

/// Pre-load all external resources referenced by the given XML content.
/// This is necessary for WASM where we cannot block on async operations.
pub(crate) async fn preload_external_resources(
    xml_content: &[u8],
    load_context: &mut LoadContext<'_>,
) -> HashMap<PathBuf, Vec<u8>> {
    let mut cache = HashMap::default();
    let paths = extract_external_paths(xml_content);
    let asset_path = load_context.path().path().to_path_buf();
    let asset_parent = asset_path.parent().map(|p| p.to_path_buf());

    for relative_path in paths {
        // Resolve the relative path against the asset's directory for loading
        let resolved_path = resolve_relative_path(&asset_path, &relative_path);

        // Tiled will look up the path as: parent_dir/relative_path (e.g., "maps/../tilesets/foo.tsx")
        // So we need to store with that key, not the normalized path
        let cache_key = if let Some(ref parent) = asset_parent {
            parent.join(&relative_path)
        } else {
            relative_path.clone()
        };

        match load_context.read_asset_bytes(resolved_path.clone()).await {
            Ok(bytes) => {
                // Recursively check for nested external references (e.g., templates in tilesets)
                let nested = extract_external_paths(&bytes);
                for nested_relative in nested {
                    // Resolve nested paths relative to the tileset's location for loading
                    let nested_resolved = resolve_relative_path(&resolved_path, &nested_relative);
                    // Cache key for nested: tileset's parent dir + nested relative path
                    let nested_cache_key = if let Some(tileset_parent) = resolved_path.parent() {
                        tileset_parent.join(&nested_relative)
                    } else {
                        nested_relative.clone()
                    };
                    if !cache.contains_key(&nested_cache_key) {
                        if let Ok(nested_bytes) =
                            load_context.read_asset_bytes(nested_resolved).await
                        {
                            cache.insert(nested_cache_key, nested_bytes);
                        }
                    }
                }
                // Store with the key that Tiled will use for lookup
                cache.insert(cache_key, bytes);
            }
            Err(e) => {
                log::warn!(
                    "Failed to pre-load external resource '{}': {}",
                    relative_path.display(),
                    e
                );
            }
        }
    }

    cache
}
