//! Debug plugin for visualizing tile indices in Bevy Tiled maps.
//!
//! This module provides a plugin and configuration for displaying the `bevy_ecs_tilemap` tile index ([`TilePos`])
//! above each tile in your map. This is useful for debugging tile placement, grid alignment, and verifying
//! that tiles are correctly positioned and indexed within your Tiled maps.
//!
//! When enabled, the plugin spawns a [`Text2d`] entity above every tile, showing its [`TilePos`] coordinates.

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
/// Enabling this plugin will insert a [`Text2d`] component into every tile entity to display its [`TilePos`] coordinates.
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
            .add_systems(Update, draw_tile_infos);
    }
}

fn draw_tile_infos(
    config: Res<TiledDebugTilesConfig>,
    assets: Res<Assets<TiledMapAsset>>,
    map_query: Query<(&TiledMap, &TiledMapStorage, &TilemapAnchor)>,
    tilemap_query: Query<&GlobalTransform, With<TiledTilemap>>,
    tile_query: Query<(&TilePos, &ChildOf), With<TiledTile>>,
    mut gizmos: Gizmos,
) {
    for (tiled_map, storage, anchor) in map_query.iter() {
        let Some(map_asset) = assets.get(&tiled_map.0) else {
            continue;
        };

        for (_, entities) in storage.tiles() {
            for entity in entities {
                let Ok((tile_pos, child_of)) = tile_query.get(*entity) else {
                    continue;
                };
                let Some(tile) = storage.get_tile(&map_asset.map, *entity) else {
                    continue;
                };

                // Compute the tile's world position (center) relative to its tilemap
                let tile_rel_pos =
                    map_asset.tile_relative_position(tile_pos, &tile_size(&tile), anchor);

                // To get the tile's global transform, combine with the parent tilemap's transform
                let Ok(parent_transform) = tilemap_query.get(child_of.parent()) else {
                    continue;
                };
                let tile_transform =
                    *parent_transform * Transform::from_translation(tile_rel_pos.extend(0.));

                gizmos.text_2d(
                    Isometry2d::from_translation(tile_transform.translation().truncate()),
                    &format!("{}x{}", tile_pos.x, tile_pos.y),
                    config.text_size,
                    Vec2::ZERO,
                    config.text_color,
                );
            }
        }
    }
}
