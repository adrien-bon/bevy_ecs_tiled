//! Debugging tools for bevy_ecs_tiled.
//!
//! This module provides plugins and utilities to help visualize and debug Tiled maps and worlds
//! in your Bevy application. Enable the `debug` feature to use these plugins, which include
//! gizmo overlays for objects, tiles, world chunks, maps bounding boxes, and axes.

pub mod axis;
pub mod map_aabb;
pub mod objects;
pub mod tiles;
#[cfg(feature = "world")]
pub mod world_chunk;

use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    prelude::*,
};

/// [`GizmoConfigGroup`] used by all `bevy_ecs_tiled` debug plugins.
///
/// All debug gizmos are drawn using this group, so they can be configured independently
/// from other gizmos of your application, using the [`GizmoConfigStore`] resource.
/// For instance, you can change line width or hide all debug gizmos at once:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn hide_debug_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
///     let (config, _) = config_store.config_mut::<TiledDebugGizmos>();
///     config.enabled = false;
///     config.line.width = 1.;
/// }
/// ```
///
/// When debug gizmos are disabled, debug systems do not run at all.
#[derive(Default, Reflect, GizmoConfigGroup, Debug, Clone, Copy)]
#[reflect(Default)]
pub struct TiledDebugGizmos;

/// Run condition returning `true` if [`TiledDebugGizmos`] are enabled.
///
/// Used by all `bevy_ecs_tiled` debug systems: they do not run when debug gizmos are disabled.
pub fn tiled_debug_gizmos_enabled(config_store: Res<GizmoConfigStore>) -> bool {
    config_store.config::<TiledDebugGizmos>().0.enabled
}

/// Plugin to toggle the display of all `bevy_ecs_tiled` debug gizmos using a keyboard key.
///
/// By default, no key is set and this plugin does nothing.
/// Use [`PluginGroupBuilder::set`] to choose a key when using [`TiledDebugPluginGroup`]:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugPluginGroup.set(TiledDebugTogglePlugin {
///         key: Some(KeyCode::F2),
///     }));
/// ```
#[derive(Default, Copy, Clone, Debug)]
pub struct TiledDebugTogglePlugin {
    /// Key toggling debug gizmos display. If [`None`], debug gizmos cannot be toggled using the keyboard.
    pub key: Option<KeyCode>,
}

impl Plugin for TiledDebugTogglePlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<TiledDebugGizmos>();
        if let Some(key) = self.key {
            app.add_systems(
                Update,
                move |keyboard: Res<ButtonInput<KeyCode>>,
                      mut config_store: ResMut<GizmoConfigStore>| {
                    if keyboard.just_pressed(key) {
                        let (config, _) = config_store.config_mut::<TiledDebugGizmos>();
                        config.enabled = !config.enabled;
                    }
                },
            );
        }
    }
}

/// This [`PluginGroup`] contains all debug plugins from `bevy_ecs_tiled`.
///
/// It can be used to easily turn on all debug informations :
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugPluginGroup);
/// ```
///
/// Use [`TiledDebugTogglePlugin`] to toggle debug gizmos display using a keyboard key:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugPluginGroup.set(TiledDebugTogglePlugin {
///         key: Some(KeyCode::F2),
///     }));
/// ```
#[derive(Default, Copy, Clone, Debug)]
pub struct TiledDebugPluginGroup;

impl PluginGroup for TiledDebugPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        let group = PluginGroupBuilder::start::<Self>()
            .add(objects::TiledDebugObjectsPlugin::default())
            .add(tiles::TiledDebugTilesPlugin::default());
        #[cfg(feature = "world")]
        let group = group.add(world_chunk::TiledDebugWorldChunkPlugin::default());
        group
            .add(map_aabb::TiledDebugMapAabbPlugin::default())
            .add(axis::TiledDebugAxisPlugin)
            .add(TiledDebugTogglePlugin::default())
    }
}
