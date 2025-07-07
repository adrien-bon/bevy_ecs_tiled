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

#[allow(clippy::type_complexity)]
fn draw_tile_infos(
    mut commands: Commands,
    config: Res<TiledDebugTilesConfig>,
    tiles_query: Query<(Entity, &ChildOf, &TilePos), (With<TiledTile>, Without<Text2d>)>,
    layer_query: Query<
        (
            &TilemapType,
            &TilemapSize,
            &TilemapTileSize,
            &TilemapGridSize,
            &TilemapAnchor,
        ),
        With<TiledTilemap>,
    >,
) {
    for (entity, child_of, tile_pos) in tiles_query.iter() {
        let Ok((map_type, map_size, tile_size, grid_size, anchor)) =
            layer_query.get(child_of.parent())
        else {
            continue;
        };
        let pos = tile_pos.center_in_world(map_size, grid_size, tile_size, map_type, anchor);
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
