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
                // Tiled may reach the same nested resource through a different lexical path.
                // Normalize both cache keys and lookups so equivalent paths share an entry.
                if let Some(data) = self.cache.get(&normalize_path(path)) {
                    return Ok(Box::new(Cursor::new(data.clone())));
                }
                return Err(IoError::new(
                    ErrorKind::NotFound,
                    format!(
                        "External tileset/template '{}' not found in cache. \
                        For WASM builds, all external tilesets must be pre-loaded.",
                        path.display()
                    ),
                ));
            }
        }
        Ok(Box::new(Cursor::new(self.bytes.clone())))
    }
}

/// Extract external tileset/template paths from TMX/TSX/TX XML content.
/// This does a simple regex-free parse to find source and template attributes.
pub(crate) fn extract_external_paths(xml_content: &[u8]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let content = String::from_utf8_lossy(xml_content);

    for line in content.lines() {
        for marker in ["source=\"", "template=\""] {
            if let Some(start) = line.find(marker) {
                let rest = &line[start + marker.len()..];
                if let Some(end) = rest.find('"') {
                    let path = &rest[..end];
                    if path.ends_with(".tsx") || path.ends_with(".tx") {
                        paths.push(PathBuf::from(path));
                    }
                }
            }
        }
    }

    paths
}

/// Normalize `.` and `..` components without accessing the filesystem.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            component => components.push(component),
        }
    }
    components.iter().collect()
}

/// Resolve a relative path against the asset's parent directory.
/// For example, if the asset is at `maps/01_first_street.tmx` and the
/// relative path is `../tilesets/kilowatt_tiles.tsx`, this returns
/// `tilesets/kilowatt_tiles.tsx`.
fn resolve_relative_path(asset_path: &Path, relative_path: &Path) -> PathBuf {
    normalize_path(&asset_path.parent().map_or_else(
        || relative_path.to_path_buf(),
        |parent| parent.join(relative_path),
    ))
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

    for relative_path in paths {
        // Resolve the relative path against the asset's directory for loading
        let resolved_path = resolve_relative_path(&asset_path, &relative_path);

        let cache_key = resolved_path.clone();

        match load_context.read_asset_bytes(resolved_path.clone()).await {
            Ok(bytes) => {
                // Recursively check for nested external references (e.g., templates in tilesets)
                let nested = extract_external_paths(&bytes);
                for nested_relative in nested {
                    // Resolve nested paths relative to the tileset's location for loading
                    let nested_resolved = resolve_relative_path(&resolved_path, &nested_relative);
                    let nested_cache_key = nested_resolved.clone();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nested_template_from_equivalent_normalized_cache_path() {
        let template_path = PathBuf::from("../templates/tree_base_collision.tx");
        assert_eq!(
            extract_external_paths(br#"<object template="../templates/tree_base_collision.tx"/>"#),
            vec![template_path]
        );

        let expected = b"nested template";
        let cache = HashMap::from([(
            PathBuf::from("tiled/templates/tree_base_collision.tx"),
            expected.to_vec(),
        )]);
        let mut reader = BytesResourceReader::new(b"", &cache);
        let mut resource = tiled::ResourceReader::read_from(
            &mut reader,
            Path::new("tiled/grassland/../tilesets/../templates/tree_base_collision.tx"),
        )
        .expect("equivalent external resource paths should share one cache entry");
        let mut actual = Vec::new();
        resource.read_to_end(&mut actual).unwrap();

        assert_eq!(actual, expected);
    }
}
