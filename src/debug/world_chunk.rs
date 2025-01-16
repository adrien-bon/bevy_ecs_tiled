use crate::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};

#[derive(Resource, Clone)]
pub struct TiledDebugWorldChunkConfig {
    /// Color of the `arrow_2d` [Gizmos]
    pub color: Color,
}

impl Default for TiledDebugWorldChunkConfig {
    fn default() -> Self {
        Self {
            color: bevy::prelude::Color::Srgba(RED),
        }
    }
}

#[derive(Default, Clone)]
pub struct TiledDebugWorldChunkPlugin(pub TiledDebugWorldChunkConfig);
impl Plugin for TiledDebugWorldChunkPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.0.clone())
            .add_systems(Update, draw_debug_rect);
    }
}

fn draw_debug_rect(
    camera_query: Query<&Transform, (With<Camera>, Changed<Transform>)>,
    world_query: Query<&TiledWorldSettings, With<TiledWorldMarker>>,
    config: Res<TiledDebugWorldChunkConfig>,
    mut gizmos: Gizmos,
) {
    for world_settings in world_query.iter() {
        let Some(chunking) = world_settings.chunking else {
            continue;
        };
        for camera_transform in camera_query.iter() {
            let position = Vec2::new(
                camera_transform.translation.x,
                camera_transform.translation.y,
            );
            gizmos.rect_2d(
                Isometry2d::from_translation(position),
                chunking * 2.,
                config.color,
            );
        }
    }
}
