//! Debug plugin for visualizing tile indices in Bevy Tiled maps.
//!
//! This module provides a plugin and configuration for displaying the `bevy_ecs_tilemap` tile index ([`TilePos`])
//! above each tile in your map. This is useful for debugging tile placement, grid alignment, and verifying
//! that tiles are correctly positioned and indexed within your Tiled maps.
//!
//! When enabled, the plugin draws a text above every rendered tile using [`Gizmos`], showing its [`TilePos`] coordinates.

use super::tiled_debug_gizmos_enabled;
use crate::prelude::*;
use bevy::{color::palettes::css::FUCHSIA, prelude::*};

/// Configuration for the [`TiledDebugTilesPlugin`].
///
/// Allows customization of the appearance and placement of the tile index text.
#[derive(Resource, Reflect, Copy, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledDebugTilesConfig {
    /// [`Color`] of the tile index text.
    pub text_color: Color,
    /// Font size of the tile index text.
    pub text_size: f32,
}

impl Default for TiledDebugTilesConfig {
    fn default() -> Self {
        Self {
            text_color: bevy::prelude::Color::Srgba(FUCHSIA),
            text_size: 10.,
        }
    }
}

/// Debug [`Plugin`] for visualizing tile indices in Bevy Tiled maps.
///
/// Enabling this plugin will draw, every frame, the [`TilePos`] coordinates of every rendered tile using [`Gizmos`].
/// This is helpful for debugging tile placement and grid alignment in your Tiled maps.
///
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugTilesPlugin::default());
/// ```
///
/// You can customize the appearance of the text using [`TiledDebugTilesConfig`].
#[derive(Default, Clone, Copy, Debug)]
pub struct TiledDebugTilesPlugin(pub TiledDebugTilesConfig);

impl Plugin for TiledDebugTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<TiledDebugTilesConfig>()
            .insert_resource(self.0)
            .init_gizmo_group::<TiledDebugGizmos>()
            .add_systems(Update, draw_tile_infos.run_if(tiled_debug_gizmos_enabled));
    }
}

fn draw_tile_infos(
    config: Res<TiledDebugTilesConfig>,
    maps: TiledMapHelper,
    tilemap_query: Query<&GlobalTransform, With<TiledTilemap>>,
    tile_query: Query<(Entity, &TilePos, &TilemapId), With<TiledTile>>,
    mut gizmos: Gizmos<TiledDebugGizmos>,
) {
    for (tile, tile_pos, tilemap_id) in tile_query.iter() {
        // Compute the tile's world position (center) relative to its tilemap
        let Some(tile_rel_pos) = maps.tile_relative_position(tile) else {
            continue;
        };

        // To get the tile's global transform, combine with the tilemap's transform
        let Ok(tilemap_transform) = tilemap_query.get(tilemap_id.0) else {
            continue;
        };
        let tile_transform =
            *tilemap_transform * Transform::from_translation(tile_rel_pos.extend(0.));

        gizmos.text_2d(
            Isometry2d::from_translation(tile_transform.translation().truncate()),
            &format!("{}x{}", tile_pos.x, tile_pos.y),
            config.text_size,
            Vec2::ZERO,
            config.text_color,
        );
    }
}
