//! Handles all logic related to Tiled custom properties.
//!
//! This module is only available when the `user_properties` feature is enabled.
//! It provides mechanisms for exporting, loading, and managing user-defined properties
//! that can be attached to Tiled maps, objects, and tiles. See the [associated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/user_properties.rs)
//! or the [dedicated book section](https://adrien-bon.github.io/bevy_ecs_tiled/guides/properties.html) for more information.

pub(crate) mod command;
pub(crate) mod export;
pub(crate) mod load;
pub(crate) mod types_json;

use crate::prelude::*;
use bevy::prelude::*;
use std::{
    fs::File,
    io::{BufWriter, Write},
    ops::Deref,
    path::Path,
};

/// Export Tiled types to the given path.
///
/// Only types whose path matches the provided [`TiledFilter`] are exported.
/// To export all types, use [`TiledFilter::All`].
///
/// # Errors
/// Returns an error if the file cannot be created or written.
pub fn export_types(
    reg: &AppTypeRegistry,
    path: impl AsRef<Path>,
    filter: &TiledFilter,
) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    let registry = export::TypeExportRegistry::from_registry(reg.read().deref(), filter);
    serde_json::to_writer_pretty(&mut writer, &registry.to_vec())?;
    writer.flush()
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Startup,
        |reg: Res<AppTypeRegistry>, config: Res<TiledPluginConfig>| {
            if let Some(path) = &config.tiled_types_export_file {
                info!("Export Tiled types to '{:?}'", &path);
                if let Err(e) = export_types(&reg, path, &config.tiled_types_filter) {
                    error!("Failed to export Tiled types to '{:?}': {e}", &path);
                }
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_types_to_invalid_path_returns_error() {
        let registry = AppTypeRegistry::default();
        let path = std::env::temp_dir()
            .join("bevy_ecs_tiled_missing_directory")
            .join("types.json");
        assert!(export_types(&registry, path, &TiledFilter::All).is_err());
    }
}
