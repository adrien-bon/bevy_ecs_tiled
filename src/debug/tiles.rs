use crate::prelude::*;
use bevy::{color::palettes::css::FUCHSIA, prelude::*};
use bevy_ecs_tilemap::{map::{TilemapGridSize, TilemapType}, tiles::TilePos};

#[derive(Resource, Clone)]
pub struct TiledDebugTilesConfig {
    pub color: Color,
    pub font: TextFont,
    pub z_offset: f32,
}

impl Default for TiledDebugTilesConfig {
    fn default() -> Self {
        Self {
            color: bevy::prelude::Color::Srgba(FUCHSIA),
            font: TextFont {
                font_size: 10.0,
                ..default()
            },
            z_offset: 500.,
        }
    }
}

#[derive(Default, Clone)]
pub struct TiledDebugTilesPlugin(pub TiledDebugTilesConfig);
impl Plugin for TiledDebugTilesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.0.clone())
            .add_systems(Update, draw_tile_infos);
    }
}

#[allow(clippy::type_complexity)]
fn draw_tile_infos(
    mut commands: Commands,
    config: Res<TiledDebugTilesConfig>,
    tiles_query: Query<(Entity, &Parent, &TilePos), (With<TiledMapTile>, Without<Text2d>)>,
    layer_query: Query<(&TilemapType, &TilemapGridSize), With<TiledMapTileLayerForTileset>>,
) {
    for (entity, parent, tile_pos) in tiles_query.iter() {
        let Ok((map_type, grid_size)) = layer_query.get(parent.get()) else {
            continue;
        };
        let pos = tile_pos.center_in_world(grid_size, map_type);
        commands.entity(entity).insert((
            Text2d::new(format!("{}x{}", tile_pos.x, tile_pos.y)),
            TextColor(config.color),
            config.font.clone(),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_translation(Vec3::new(
                pos.x,
                pos.y,
                config.z_offset,
            )),
        ));
    }
}