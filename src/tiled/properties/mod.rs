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
use std::{fs::File, io::BufWriter, ops::Deref, path::Path};

/// Export a Tiled types to the given path.
///
/// The predicate determines whether a symbol is exported. To export all
/// symbols, one can provide a blanket yes predicate, e.g. `|_| true`.
pub fn export_types(
    reg: &AppTypeRegistry,
    path: impl AsRef<Path>,
    predicate: impl Fn(&str) -> bool,
) {
    let file = File::create(path).unwrap();
    let writer = BufWriter::new(file);
    let registry = export::TypeExportRegistry::from_registry(reg.read().deref());
    let mut list = registry.to_vec();
    list.retain(|v| predicate(&v.name));
    serde_json::to_writer_pretty(writer, &list).unwrap();
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Startup,
        |reg: Res<AppTypeRegistry>, config: Res<TiledPluginConfig>| {
            if let Some(path) = &config.tiled_types_export_file {
                info!("Export Tiled types to '{:?}'", &path);
                export_types(&reg, path, |_| true);
            }
        },
    );
}
