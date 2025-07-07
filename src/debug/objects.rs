//! Debug plugin for visualizing Tiled objects in Bevy.
//!
//! This module provides a plugin and configuration for displaying Bevy [`Gizmos`] at the positions of Tiled objects.
//! It is useful for debugging object placement, orientation, and map integration in your Tiled maps.
//!
//! When enabled, the plugin draws a 2D arrow gizmo at each Tiled object's position, allowing you to easily
//! verify where objects are spawned in your world.

use crate::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};

/// Configuration for the [`TiledDebugObjectsPlugin`].
///
/// Contains settings to customize how the `arrow_2d` [`Gizmos`] will appear for each Tiled object.
#[derive(Resource, Reflect, Copy, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledDebugObjectsConfig {
    /// Color of the `arrow_2d` [`Gizmos`].
    pub color: Color,
    /// Length and direction of the `arrow_2d` [`Gizmos`].
    pub arrow_length: Vec2,
}

impl Default for TiledDebugObjectsConfig {
    fn default() -> Self {
        Self {
            color: bevy::prelude::Color::Srgba(RED),
            arrow_length: Vec2::new(0., 20.),
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
#[derive(Default, Copy, Clone, Debug)]
pub struct TiledDebugObjectsPlugin(pub TiledDebugObjectsConfig);

impl Plugin for TiledDebugObjectsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<TiledDebugObjectsConfig>()
            .insert_resource(self.0)
            .add_systems(Update, draw_debug_arrow);
    }
}

fn draw_debug_arrow(
    objects_query: Query<&GlobalTransform, With<TiledObject>>,
    config: Res<TiledDebugObjectsConfig>,
    mut gizmos: Gizmos,
) {
    for transform in objects_query.iter() {
        let pos = Vec2::new(transform.translation().x, transform.translation().y);
        gizmos.arrow_2d(pos + config.arrow_length, pos, config.color);
    }
}
