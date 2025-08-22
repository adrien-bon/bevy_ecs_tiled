//! Debug plugin for visualizing Tiled objects in Bevy.
//!
//! This module provides a plugin and configuration for displaying Bevy [`Gizmos`] at the positions of Tiled objects.
//! It is especially useful for debugging object placement, orientation, and integration of Tiled maps in your Bevy game.
//!
//! When enabled, the plugin draws:
//!   - A 2D polyline gizmo outlining each Tiled object's shape.
//!   - A 2D arrow gizmo highlighting the object's position and orientation.
//!
//! This allows you to easily verify where objects are spawned and how they are oriented in your world.

use crate::prelude::*;
use bevy::{
    color::palettes::css::{BLUE, FUCHSIA, GREEN, LIME, RED, WHITE, YELLOW},
    prelude::*,
};

/// Configuration for the [`TiledDebugObjectsPlugin`].
///
/// This struct allows you to customize how the debug gizmos appear for each Tiled object.
/// - `objects_colors_list`: The list of colors used to draw arrows and outlines for objects (cycled through for each object).
/// - `arrow_length`: The length and direction of the arrow gizmo drawn at each object's position.
#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledDebugObjectsConfig {
    /// Colors used for the `arrow_2d` and outline [`Gizmos`]. Cycled for each object.
    pub objects_colors_list: Vec<Color>,
    /// Length and direction of the `arrow_2d` [`Gizmos`] drawn at each object's position.
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
/// Add this plugin to your app to display a 2D arrow and outline gizmo at the position and shape of each [`TiledObject`] entity.
/// This is helpful for debugging and verifying object placement, orientation, and geometry in your Tiled maps.
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
/// This will display an `arrow_2d` and a polyline outline [`Gizmos`] at each Tiled object's position and shape.
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
    map_query: Query<(&TiledMap, &TiledMapStorage)>,
    assets: Res<Assets<TiledMapAsset>>,
    object_query: Query<(&TiledObject, &GlobalTransform)>,
    config: Res<TiledDebugObjectsConfig>,
    mut gizmos: Gizmos,
) {
    for (tiled_map, storage) in map_query.iter() {
        let Some(map_asset) = assets.get(&tiled_map.0) else {
            continue;
        };
        // XXX: stil and issue on isometric maps for objects with a non-null rotation
        for (idx, (_, entity)) in storage.objects().enumerate() {
            if let Ok((object, transform)) = object_query.get(*entity) {
                let color = config.objects_colors_list[idx % config.objects_colors_list.len()];
                let origin = Vec2::new(transform.translation().x, transform.translation().y);
                gizmos.arrow_2d(origin + config.arrow_length, origin, color);
                let positions = object
                    .line_string(
                        transform,
                        matches!(
                            tilemap_type_from_map(&map_asset.map),
                            TilemapType::Isometric(..)
                        ),
                        &map_asset.tilemap_size,
                        &grid_size_from_map(&map_asset.map),
                        map_asset.tiled_offset,
                    )
                    .map(|ls| ls.coords().map(|c| Vec2::new(c.x, c.y)).collect::<Vec<_>>());
                if let Some(pos) = positions {
                    gizmos.linestrip_2d(pos, color);
                }
            }
        }
    }
}
