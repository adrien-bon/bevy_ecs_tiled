//! Debug plugin for visualizing physics collider chunks of Tiled tiles layers.
//!
//! This module provides a plugin and configuration for displaying the bounding box of each
//! [`TiledColliderChunk`], ie. each group of colliders of a tiles layer when its colliders
//! are split in chunks (see [`TiledPhysicsSettings::tiles_chunk_size`]).
//!
//! It is only available when both the `debug` and `physics` features are enabled.

use super::tiled_debug_gizmos_enabled;
use crate::prelude::*;
use ::geo::BoundingRect;
use bevy::{color::palettes::css::ORANGE, prelude::*};

/// Configuration for the [`TiledDebugColliderChunksPlugin`].
#[derive(Resource, Reflect, Copy, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledDebugColliderChunksConfig {
    /// [`Color`] of the collider chunks bounding boxes.
    pub color: Color,
}

impl Default for TiledDebugColliderChunksConfig {
    fn default() -> Self {
        Self {
            color: Color::Srgba(ORANGE),
        }
    }
}

/// Debug [`Plugin`] for visualizing physics collider chunks of Tiled tiles layers.
///
/// Enabling this plugin will draw, every frame, the bounding box of the colliders of each
/// [`TiledColliderChunk`] using [`Gizmos`]: chunks which do not hold any collider are not displayed.
/// Nothing is drawn for tiles layers whose colliders are not split in chunks, which is the default.
///
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugColliderChunksPlugin::default());
/// ```
#[derive(Default, Clone, Copy, Debug)]
pub struct TiledDebugColliderChunksPlugin(pub TiledDebugColliderChunksConfig);

impl Plugin for TiledDebugColliderChunksPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<TiledDebugColliderChunksConfig>()
            .insert_resource(self.0)
            .init_gizmo_group::<TiledDebugGizmos>()
            .add_systems(
                Update,
                draw_collider_chunks.run_if(tiled_debug_gizmos_enabled),
            );
    }
}

fn draw_collider_chunks(
    config: Res<TiledDebugColliderChunksConfig>,
    chunks_query: Query<(&GlobalTransform, &TiledColliderPolygons), With<TiledColliderChunk>>,
    mut gizmos: Gizmos<TiledDebugGizmos>,
) {
    for (transform, polygons) in chunks_query.iter() {
        for rect in polygons
            .iter()
            .filter_map(|polygon| polygon.bounding_rect())
        {
            let (min, max) = (rect.min(), rect.max());
            // Polygons are relative to the chunk entity, which can be rotated
            let corners = [
                Vec2::new(min.x, min.y),
                Vec2::new(max.x, min.y),
                Vec2::new(max.x, max.y),
                Vec2::new(min.x, max.y),
                Vec2::new(min.x, min.y),
            ]
            .map(|corner| transform.transform_point(corner.extend(0.)).truncate());
            gizmos.linestrip_2d(corners, config.color);
        }
    }
}
