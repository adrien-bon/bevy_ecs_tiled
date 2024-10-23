//! This module contains some tools to help you debug your application.
//!
use crate::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};
use bevy_ecs_tilemap::tiles::TilePos;

/// Debug [Gizmos] configuration
///
/// Contains some settings to customize how the `arrow_2d` [Gizmos] will appear.
#[derive(Resource, Clone)]
pub struct TiledMapGizmosConfig {
    /// Color of the `arrow_2d` [Gizmos]
    pub color: Color,
    /// Length of the `arrow_2d` [Gizmos]
    pub arrow_length: Vec2,
}

impl Default for TiledMapGizmosConfig {
    fn default() -> Self {
        Self {
            color: bevy::prelude::Color::Srgba(RED),
            arrow_length: Vec2::new(0., 20.),
        }
    }
}

/// `bevy_ecs_tiled` debug `Plugin`
///
/// In case you want to debug your application, you should add this plugin:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledMapDebugPlugin::default());
/// ```
///
/// This will display an `arrow_2d` [Gizmos] where your objects are.
///
#[derive(Default, Clone)]
pub struct TiledMapDebugPlugin {
    /// Debug gizmos configuration
    pub gizmos_config: TiledMapGizmosConfig,
}

impl Plugin for TiledMapDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.gizmos_config.clone())
            .add_systems(Update, (draw_debug_arrow, draw_debug_pos));
    }
}

fn draw_debug_arrow(
    q_objects: Query<&GlobalTransform, With<TiledMapObject>>,
    config: Res<TiledMapGizmosConfig>,
    mut gizmos: Gizmos,
) {
    for transform in q_objects.iter() {
        let pos = Vec2::new(transform.translation().x, transform.translation().y);
        gizmos.arrow_2d(pos + config.arrow_length, pos, config.color);
        
    }
}

#[derive(Component)]
struct DebugPos;
fn draw_debug_pos(mut commands: Commands, q_pos: Query<(Entity, &TilePos), Without<DebugPos>>) {
    for (e, tile_pos) in q_pos.iter() {
        commands
            .spawn(Text2dBundle {
                text: Text::from_section(
                    format!("{0}, {1}", tile_pos.x, tile_pos.y),
                    TextStyle {
                        color: Color::srgb(0.25, 0.75, 0.25),
                        font_size: 12.0,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0., 0., 100.), // Set it to top
                ..default()
            })
            .set_parent(e);
        commands.entity(e).insert(DebugPos);
    }
}
