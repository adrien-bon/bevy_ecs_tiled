//! Debug plugin for visualizing Tiled objects in Bevy.
//!
//! This module provides a plugin and configuration for displaying Bevy [`Gizmos`] at the positions of Tiled objects.
//! It is useful for debugging object placement, orientation, and map integration in your Tiled maps.
//!
//! When enabled, the plugin draws a 2D arrow gizmo at each Tiled object's position, allowing you to easily
//! verify where objects are spawned in your world.

use crate::prelude::*;
use bevy::{
    color::palettes::css::{BLUE, FUCHSIA, GREEN, LIME, RED, WHITE, YELLOW},
    prelude::*,
};

/// Configuration for the [`TiledDebugObjectsPlugin`].
///
/// Contains settings to customize how the `arrow_2d` [`Gizmos`] will appear for each Tiled object.
#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledDebugObjectsConfig {
    /// Color of the `arrow_2d` [`Gizmos`].
    pub objects_colors_list: Vec<Color>,
    /// Length and direction of the `arrow_2d` [`Gizmos`].
    pub arrow_length: Vec2,
}

impl Default for TiledDebugObjectsConfig {
    fn default() -> Self {
        Self {
            arrow_length: Vec2::new(-15., 15.),
            objects_colors_list: vec![
                Color::from(RED),
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

/// Debug [`Plugin`] for visualizing Tiled objects in Bevy.
///
/// Add this plugin to your app to display a 2D arrow gizmo at the position of each [`TiledObject`] entity.
/// This is helpful for debugging and verifying object placement in your Tiled maps.
///
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugObjectsPlugin::default());
/// ```
///
/// This will display an `arrow_2d` [`Gizmos`] at each Tiled object's position.
#[derive(Default, Clone, Debug)]
pub struct TiledDebugObjectsPlugin(pub TiledDebugObjectsConfig);

impl Plugin for TiledDebugObjectsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<TiledDebugObjectsConfig>()
            .insert_resource(self.0.clone())
            .add_systems(Update, draw_debug_gizmos);
    }
}

fn draw_debug_gizmos(
    objects_query: Query<(&TiledObject, &GlobalTransform)>,
    config: Res<TiledDebugObjectsConfig>,
    mut gizmos: Gizmos,
) {
    for (idx, (object, transform)) in objects_query.iter().enumerate() {
        let color = config.objects_colors_list[idx % config.objects_colors_list.len()];
        let origin = Vec2::new(transform.translation().x, transform.translation().y);
        match object {
            TiledObject::Point | TiledObject::Text => {}
            TiledObject::Rectangle { width, height } | TiledObject::Tile { width, height } => {
                gizmos.rect_2d(
                    object.isometry_2d(transform),
                    Vec2::new(*width, *height),
                    color,
                );
            }
            TiledObject::Ellipse { width, height } => {
                gizmos.ellipse_2d(
                    object.isometry_2d(transform),
                    Vec2::new(*width / 2., *height / 2.),
                    color,
                );
            }
            TiledObject::Polygon { vertices: _ } => {
                let mut positions = object.vertices(transform);
                positions.push(positions[0]); // Close the polygon
                gizmos.linestrip_2d(positions, color);
            }
            TiledObject::Polyline { vertices: _ } => {
                let positions = object.vertices(transform);
                gizmos.linestrip_2d(positions, color);
            }
        }

        gizmos.arrow_2d(origin + config.arrow_length, origin, color);
    }
}
