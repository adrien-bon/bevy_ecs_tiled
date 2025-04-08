//! Debug plugin for tiles
//!
//! Display the `bevy_ecs_tilemap` index, ie. [TilePos] on each tile

use crate::prelude::*;
use bevy::{color::palettes::css::FUCHSIA, prelude::*};
use bevy_ecs_tilemap::{
    map::{TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType},
    prelude::TilemapAnchor,
    tiles::TilePos,
};

/// Configuration for the [TiledDebugTilesPlugin]
#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledDebugTilesConfig {
    /// [Color] of the tile index text
    pub color: Color,
    /// [TextFont] of the tile index text
    pub font: TextFont,
    /// Absolute Z-axis offset of the tile index text
    pub z_offset: f32,
    /// Scale to apply on the tile index text
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

/// `bevy_ecs_tiled` debug [Plugin] for tiles
///
/// Enabling this plugin will spawn a [Text2d] above every tile to display their [TilePos] :
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugTilesPlugin::default());
/// ```
///
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
    tiles_query: Query<(Entity, &Parent, &TilePos), (With<TiledMapTile>, Without<Text2d>)>,
    layer_query: Query<
        (
            &TilemapType,
            &TilemapSize,
            &TilemapTileSize,
            &TilemapGridSize,
            &TilemapAnchor,
        ),
        With<TiledMapTileLayerForTileset>,
    >,
) {
    for (entity, parent, tile_pos) in tiles_query.iter() {
        let Ok((map_type, map_size, tile_size, grid_size, anchor)) = layer_query.get(parent.get())
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
