//! This example cycle through all four kinds of hexagonal maps and display debug informations about Tiled objects.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .add_plugins(PhysicsDebugPlugin::default())
        // Enable debug informations about Tiled objects
        .add_plugins(TiledMapDebugPlugin::default())
        .add_plugins(helper::HelperPlugin)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TiledMapBundle {
        tiled_map: asset_server.load("tiledtest_y_odd.tmx"),
        tiled_settings: TiledMapSettings {
            physics_backend: PhysicsBackend::Avian,
            ..default()
        },
        ..default()
    });
}
