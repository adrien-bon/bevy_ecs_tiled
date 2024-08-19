//! This example shows a finite orthogonal map with multiple external tilesets.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(helper::HelperPlugin)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // For simplicity sake, we use two tilesets which actually use the same images
    // However, we can verify with the inspector that the map actually use tiles
    // from both tilesets
    let map_handle: Handle<TiledMap> = asset_server.load("multiple_tilesets.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}
