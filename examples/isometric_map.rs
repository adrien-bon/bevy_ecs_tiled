//! This example shows a finite isometric map with an external tileset.

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
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<TiledMap> = asset_server.load("isometric_map.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        render_settings: TilemapRenderSettings {
            // bevy_ecs_tilemap provide the 'y_sort' parameter to
            // sort chunks using their y-axis position during rendering.
            // However, it applies to whole chunks, not individual tile,
            // so we have to force the chunk size to be exactly one tile
            render_chunk_size: UVec2::new(1, 1),
            y_sort: true,
        },
        tiled_settings: TiledMapSettings {
            // Not related to current example, but center the map
            map_positioning: MapPositioning::Centered,
            ..default()
        },
        ..Default::default()
    });
}
