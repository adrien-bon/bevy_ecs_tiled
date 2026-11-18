//! Debug plugin for world streaming in `bevy_ecs_tiled`.
//!
//! This module provides a plugin and configuration for visualizing world render chunks and map boundaries
//! using Bevy [`Gizmos`]. It is useful for debugging how your Tiled worlds are chunked and rendered, and for
//! verifying the alignment and boundaries of maps within a world.
//!
//! When enabled, the plugin draws a colored `rect_2d` gizmo for each world render chunk (centered on the camera)
//! and for each map boundary.

use super::tiled_debug_gizmos_enabled;
use crate::prelude::*;
use bevy::{
    color::palettes::css::{BLUE, FUCHSIA, GREEN, LIME, RED, WHITE, YELLOW},
    prelude::*,
    shape::BoundingVolume,
};

/// Configuration for the [`TiledDebugWorldStreamingPlugin`].
///
/// Allows customization of the appearance of the `rect_2d` [`Gizmos`] for world render chunks and map boundaries.
#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledDebugWorldStreamingConfig {
    /// [`Color`] of the `rect_2d` [`Gizmos`] for the world rendering chunk.
    ///
    /// If [`None`], the world rendering chunk will not be displayed.
    pub streaming_area_color: Option<Color>,
    /// List of [`Color`]s for the `rect_2d` [`Gizmos`] representing map boundaries.
    ///
    /// The plugin cycles through this list for each map, so adjacent maps can be visually distinguished.
    /// If the list is empty, no map boundaries will be displayed.
    pub maps_colors_list: Vec<Color>,
}

impl Default for TiledDebugWorldStreamingConfig {
    fn default() -> Self {
        Self {
            streaming_area_color: Some(Color::from(RED)),
            maps_colors_list: vec![
                Color::from(FUCHSIA),
                Color::from(WHITE),
                Color::from(BLUE),
                Color::from(GREEN),
                Color::from(YELLOW),
                Color::from(LIME),
            ],
        }
    }
}

/// Debug [`Plugin`] for visualizing world streaming area and map boundaries in Tiled worlds.
///
/// Add this plugin to your app to display colored rectangles for each world render chunk (centered on the camera)
/// and for each map boundary. This is helpful for debugging streaming parameters and map alignment in your Tiled worlds.
///
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugWorldStreamingPlugin::default());
/// ```
#[derive(Default, Clone, Debug)]
pub struct TiledDebugWorldStreamingPlugin(pub TiledDebugWorldStreamingConfig);

impl Plugin for TiledDebugWorldStreamingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<TiledDebugWorldStreamingConfig>()
            .insert_resource(self.0.clone())
            .init_gizmo_group::<TiledDebugGizmos>()
            .add_systems(
                Update,
                (draw_camera_rect, draw_maps_rect).run_if(tiled_debug_gizmos_enabled),
            );
    }
}

fn draw_camera_rect(
    camera_query: Query<&Transform, With<Camera>>,
    world_query: Query<&TiledWorldStreaming>,
    config: Res<TiledDebugWorldStreamingConfig>,
    mut gizmos: Gizmos<TiledDebugGizmos>,
) {
    let Some(color) = config.streaming_area_color else {
        return;
    };
    for world_streaming in world_query.iter() {
        let Some(area) = world_streaming.area() else {
            continue;
        };
        for camera_transform in camera_query.iter() {
            let position = Vec2::new(
                camera_transform.translation.x,
                camera_transform.translation.y,
            );
            gizmos.rect_2d(Isometry2d::from_translation(position), area * 2., color);
        }
    }
}

fn draw_maps_rect(
    world_query: Query<(&TiledWorld, &GlobalTransform, &TilemapAnchor)>,
    world_assets: Res<Assets<TiledWorldAsset>>,
    config: Res<TiledDebugWorldStreamingConfig>,
    mut gizmos: Gizmos<TiledDebugGizmos>,
) {
    if config.maps_colors_list.is_empty() {
        return;
    }
    for (world_handle, world_transform, anchor) in world_query.iter() {
        if let Some(tiled_world) = world_assets.get(world_handle.0.id()) {
            tiled_world.for_each_map(world_transform, anchor, |idx, aabb| {
                gizmos.rect_2d(
                    Isometry2d::from_translation(aabb.center()),
                    aabb.half_size() * 2.,
                    config.maps_colors_list[idx.0 as usize % config.maps_colors_list.len()],
                );
            });
        }
    }
}
