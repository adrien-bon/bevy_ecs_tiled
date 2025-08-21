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
#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledDebugTilesConfig {
    /// [`Color`] of the tile index text.
    pub color: Color,
    /// [`TextFont`] used for the tile index text.
    pub font: TextFont,
    /// Absolute Z-axis offset for the tile index text (controls rendering order).
    pub z_offset: f32,
    /// Scale to apply to the tile index text.
    pub scale: Vec3,
}

impl Default for TiledDebugTilesConfig {
    fn default() -> Self {
        Self {
            color: bevy::prelude::Color::Srgba(FUCHSIA),
            font: TextFont::from_font_size(10.),
            z_offset: 500.,
            scale: Vec3::splat(0.5),
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
#[derive(Default, Clone, Debug)]
pub struct TiledDebugTilesPlugin(pub TiledDebugTilesConfig);

impl Plugin for TiledDebugTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<TiledDebugTilesConfig>()
            .insert_resource(self.0.clone())
            .add_systems(Update, draw_tile_infos);
    }
}

fn draw_tile_infos(
    mut commands: Commands,
    config: Res<TiledDebugTilesConfig>,
    assets: Res<Assets<TiledMapAsset>>,
    map_query: Query<(&TiledMap, &TiledMapStorage, &TilemapAnchor)>,
    tile_query: Query<(Entity, &TilePos), (With<TiledTile>, Without<Text2d>)>,
) {
    for (tiled_map, storage, anchor) in map_query.iter() {
        let Some(map_asset) = assets.get(&tiled_map.0) else {
            continue;
        };

        for (_, entities) in storage.tiles() {
            for entity in entities {
                let Ok((entity, tile_pos)) = tile_query.get(*entity) else {
                    continue;
                };
                let Some(tile) = storage.get_tile(&map_asset.map, entity) else {
                    continue;
                };

                let pos = map_asset.tile_relative_position(tile_pos, &tile_size(&tile), anchor);
                commands.entity(entity).insert((
                    Text2d::new(format!("{}x{}", tile_pos.x, tile_pos.y)),
                    TextColor(config.color),
                    config.font.clone(),
                    TextLayout::new_with_justify(JustifyText::Center),
                    Transform {
                        translation: Vec3::new(pos.x, pos.y, config.z_offset),
                        scale: config.scale,
                        ..default()
                    },
                ));
            }
        }
    }
}
