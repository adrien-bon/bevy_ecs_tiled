//! This example cycle through two kinds of isometric maps and display debug informations about Tiled objects.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(TiledMapDebugPlugin::default())
        .add_plugins(helper::HelperPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, switch_map)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut mgr = helper::assets::AssetsManager::new(&mut commands);
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        TilemapRenderSettings {
            // bevy_ecs_tilemap provide the 'y_sort' parameter to
            // sort chunks using their y-axis position during rendering.
            // However, it applies to whole chunks, not individual tile,
            // so we have to force the chunk size to be exactly one tile
            render_chunk_size: UVec2::new(1, 1),
            y_sort: true,
        },
        TiledMapSettings {
            map_positioning: MapPositioning::Centered,
            ..Default::default()
        },
        "isometric_diamond_map.tmx",
        "A finite diamond isometric map",
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        TilemapRenderSettings {
            // bevy_ecs_tilemap provide the 'y_sort' parameter to
            // sort chunks using their y-axis position during rendering.
            // However, it applies to whole chunks, not individual tile,
            // so we have to force the chunk size to be exactly one tile
            render_chunk_size: UVec2::new(1, 1),
            y_sort: true,
        },
        TiledMapSettings {
            map_positioning: MapPositioning::Centered,
            ..Default::default()
        },
        "isometric_staggered_map.tmx",
        "A finite staggered isometric map",
    ));
    commands.insert_resource(mgr);
}

fn switch_map(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mgr: ResMut<helper::assets::AssetsManager>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        mgr.cycle_map(&mut commands);
    }
}
