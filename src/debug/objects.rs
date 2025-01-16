//! Debug plugin for objects

use crate::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};

/// Configuration for the [TiledDebugObjectsPlugin]
///
/// Contains some settings to customize how the `arrow_2d` [Gizmos] will appear.
#[derive(Resource, Clone)]
pub struct TiledDebugObjectsConfig {
    /// Color of the `arrow_2d` [Gizmos]
    pub color: Color,
    /// Length of the `arrow_2d` [Gizmos]
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

/// `bevy_ecs_tiled` debug `Plugin` for objects positions.
///
/// In case you want to debug where your Tiled objects are actually spawned, you can use this plugin:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugObjectsPlugin::default());
/// ```
///
/// This will display an `arrow_2d` [Gizmos] where your objects are.
///
#[derive(Default, Clone)]
pub struct TiledDebugObjectsPlugin(pub TiledDebugObjectsConfig);
impl Plugin for TiledDebugObjectsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.0.clone())
            .add_systems(Update, draw_debug_arrow);
    }
}

fn draw_debug_arrow(
    objects_query: Query<&GlobalTransform, With<TiledMapObject>>,
    config: Res<TiledDebugObjectsConfig>,
    mut gizmos: Gizmos,
) {
    for transform in objects_query.iter() {
        let pos = Vec2::new(transform.translation().x, transform.translation().y);
        gizmos.arrow_2d(pos + config.arrow_length, pos, config.color);
    }
}
