//! Debug plugin for world chunking
//!
//! Display a Bevy [Gizmos] for each map boundary and world render chunk

use crate::prelude::*;
use bevy::{
    color::palettes::css::{BLUE, FUCHSIA, GREEN, LIME, RED, WHITE, YELLOW},
    math::bounding::BoundingVolume,
    prelude::*,
};
use bevy_ecs_tilemap::prelude::TilemapAnchor;

/// Configuration for the [TiledDebugWorldChunkPlugin]
///
/// Contains some settings to customize how the `rect_2d` [Gizmos] will appear.
#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledDebugWorldChunkConfig {
    /// [Color] of the `rect_2d` [Gizmos] for world rendering chunk
    ///
    /// If [None], will not display the world rendering chunk
    pub world_chunk_color: Option<Color>,
    /// List of [Color]s of the `rect_2d` [Gizmos] for maps boundary
    ///
    /// Will cycle through the [Vec] for each map, such as joint maps should not have the same [Color]
    /// If the [Vec] is empty, will not display any map boundary
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

/// `bevy_ecs_tiled` debug [Plugin] for world chunking
///
/// You can use this plugin to debug how your worlds are rendered and to tweak their chunking parameter :
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugWorldChunkPlugin::default());
/// ```
///
/// This will display a `rect_2d` [Gizmos] to highlight your maps boundary and another `rect_2d` for the world render chunk.
///
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
    world_query: Query<&TiledWorldChunking, With<TiledWorldMarker>>,
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
    world_query: Query<
        (&TiledWorldHandle, &GlobalTransform, &TilemapAnchor),
        With<TiledWorldMarker>,
    >,
    world_assets: Res<Assets<TiledWorld>>,
    config: Res<TiledDebugWorldChunkConfig>,
    mut gizmos: Gizmos,
) {
    if config.maps_colors_list.is_empty() {
        return;
    }
    for (world_handle, world_transform, anchor) in world_query.iter() {
        if let Some(tiled_world) = world_assets.get(world_handle.0.id()) {
            let offset = tiled_world.offset(anchor);
            crate::world::for_each_map(
                tiled_world,
                world_transform,
                offset.extend(0.0),
                |idx, aabb| {
                    gizmos.rect_2d(
                        Isometry2d::from_translation(aabb.center()),
                        aabb.half_size() * 2.,
                        config.maps_colors_list[idx % config.maps_colors_list.len()],
                    );
                },
            );
        }
    }
}
