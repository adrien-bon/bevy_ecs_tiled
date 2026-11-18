//! The main `bevy_ecs_tiled` [`Plugin`].

#[cfg(feature = "user_properties")]
use crate::prelude::*;
use bevy::prelude::*;
#[cfg(feature = "user_properties")]
use std::path::PathBuf;

/// `bevy_ecs_tiled` main [`Plugin`].
///
/// This [`Plugin`] should be added to your application to actually be able to load a Tiled map.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledPlugin::default());
/// ```
///
/// With the `user_properties` feature, the plugin can export the types registered in your application
/// to a file, at startup, so you can import them in Tiled (see the
/// [dedicated book section](https://adrien-bon.github.io/bevy_ecs_tiled/guides/properties.html)):
/// ```rust,no_run
/// # #[cfg(feature = "user_properties")]
/// # {
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new().add_plugins(TiledPlugin {
///     types_export_file: Some("tiled_types_export.json".into()),
///     types_filter: TiledFilter::from(["my_game::Player", "my_game::Enemy"]),
/// });
/// # }
/// ```
#[derive(Default, Clone, Debug)]
pub struct TiledPlugin {
    /// Path of the file Tiled types are exported to, at startup.
    ///
    /// Defaults to [`None`]: types are not exported.
    #[cfg(feature = "user_properties")]
    pub types_export_file: Option<PathBuf>,
    /// Only types whose path matches this filter are exported.
    ///
    /// Defaults to [`TiledFilter::All`].
    #[cfg(feature = "user_properties")]
    pub types_filter: TiledFilter,
}

impl Plugin for TiledPlugin {
    fn build(&self, mut app: &mut App) {
        if !app.is_plugin_added::<bevy_ecs_tilemap::TilemapPlugin>() {
            app = app.add_plugins(bevy_ecs_tilemap::TilemapPlugin);
        }

        #[cfg(feature = "user_properties")]
        if let Some(path) = &self.types_export_file {
            app.insert_resource(crate::properties::user::TiledTypesExport {
                path: path.clone(),
                filter: self.types_filter.clone(),
            });
        }

        app.add_plugins((
            crate::asset::plugin,
            crate::ecs::plugin,
            crate::properties::plugin,
            #[cfg(feature = "render")]
            crate::render::plugin,
            crate::sets::plugin,
        ));
    }
}
