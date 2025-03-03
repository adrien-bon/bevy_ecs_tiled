use crate::prelude::*;
use bevy::{
    color::palettes::css::{BLUE, FUCHSIA, GREEN, LIME, RED, WHITE, YELLOW},
    math::bounding::BoundingVolume,
    prelude::*,
};

#[derive(Resource, Clone)]
pub struct TiledDebugWorldChunkConfig {
    /// Color of the `arrow_2d` [Gizmos]
    pub camera_color: Option<Color>,
    pub maps_colors_list: Vec<Color>,
}

impl Default for TiledDebugWorldChunkConfig {
    fn default() -> Self {
        Self {
            camera_color: Some(Color::from(RED)),
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

#[derive(Default, Clone)]
pub struct TiledDebugWorldChunkPlugin(pub TiledDebugWorldChunkConfig);
impl Plugin for TiledDebugWorldChunkPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.0.clone())
            .add_systems(Update, (draw_camera_rect, draw_maps_rect));
    }
}

fn draw_camera_rect(
    camera_query: Query<&Transform, (With<Camera>, Changed<Transform>)>,
    world_query: Query<&TiledWorldChunking, With<TiledWorldMarker>>,
    config: Res<TiledDebugWorldChunkConfig>,
    mut gizmos: Gizmos,
) {
    let Some(color) = config.camera_color else {
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
        (&TiledWorldHandle, &GlobalTransform, &TiledMapAnchor),
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
            crate::world::for_each_map(tiled_world, world_transform, offset, |idx, aabb| {
                gizmos.rect_2d(
                    Isometry2d::from_translation(aabb.center()),
                    aabb.half_size() * 2.,
                    config.maps_colors_list[idx % config.maps_colors_list.len()],
                );
            });
        }
    }
}
