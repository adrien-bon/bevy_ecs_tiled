//! Debug plugin for world chunking in bevy_ecs_tiled.
//!
//! This module provides a plugin and configuration for visualizing world render chunks and map boundaries
//! using Bevy [`Gizmos`]. It is useful for debugging how your Tiled worlds are chunked and rendered, and for
//! verifying the alignment and boundaries of maps within a world.
//!
//! When enabled, the plugin draws a colored `rect_2d` gizmo for each world render chunk (centered on the camera)
//! and for each map boundary.

use crate::prelude::*;
use bevy::{
    color::palettes::css::{BLUE, FUCHSIA, GREEN, LIME, RED, WHITE, YELLOW},
    math::bounding::BoundingVolume,
    prelude::*,
};

/// Configuration for the [`TiledDebugWorldChunkPlugin`].
///
/// Allows customization of the appearance of the `rect_2d` [`Gizmos`] for world render chunks and map boundaries.
#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledDebugWorldChunkConfig {
    /// [`Color`] of the `rect_2d` [`Gizmos`] for the world rendering chunk.
    ///
    /// If [`None`], the world rendering chunk will not be displayed.
    pub world_chunk_color: Option<Color>,
    /// List of [`Color`]s for the `rect_2d` [`Gizmos`] representing map boundaries.
    ///
    /// The plugin cycles through this list for each map, so adjacent maps can be visually distinguished.
    /// If the list is empty, no map boundaries will be displayed.
    pub maps_colors_list: Vec<Color>,
}

impl Default for TiledDebugWorldChunkConfig {
    fn default() -> Self {
        Self {
            world_chunk_color: Some(Color::from(RED)),
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

/// Debug [`Plugin`] for visualizing world chunking and map boundaries in Tiled worlds.
///
/// Add this plugin to your app to display colored rectangles for each world render chunk (centered on the camera)
/// and for each map boundary. This is helpful for debugging chunking parameters and map alignment in your Tiled worlds.
///
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugWorldChunkPlugin::default());
/// ```
#[derive(Default, Clone, Debug)]
pub struct TiledDebugWorldChunkPlugin(pub TiledDebugWorldChunkConfig);

impl Plugin for TiledDebugWorldChunkPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<TiledDebugWorldChunkConfig>()
            .insert_resource(self.0.clone())
            .add_systems(Update, (draw_camera_rect, draw_maps_rect));
    }
}

fn draw_camera_rect(
    camera_query: Query<&Transform, (With<Camera>, Changed<Transform>)>,
    world_query: Query<&TiledWorldChunking>,
    config: Res<TiledDebugWorldChunkConfig>,
    mut gizmos: Gizmos,
) {
    let Some(color) = config.world_chunk_color else {
        return;
    };
    for world_chunking in world_query.iter() {
        let Some(chunking) = world_chunking.0 else {
            continue;
        };
        for camera_transform in camera_query.iter() {
            let position = Vec2::new(
                camera_transform.translation.x,
                camera_transform.translation.y,
            );
            gizmos.rect_2d(Isometry2d::from_translation(position), chunking * 2., color);
        }
    }
}

fn draw_maps_rect(
    world_query: Query<(&TiledWorld, &GlobalTransform, &TilemapAnchor)>,
    world_assets: Res<Assets<TiledWorldAsset>>,
    config: Res<TiledDebugWorldChunkConfig>,
    mut gizmos: Gizmos,
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
                    config.maps_colors_list[idx as usize % config.maps_colors_list.len()],
                );
            });
        }
    }
}
