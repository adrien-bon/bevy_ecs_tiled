//! This example shows a finite orthogonal map with an external tileset and Rapier physics.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(helper::HelperPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<TiledMap> = asset_server.load("finite.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        tiled_settings: TiledMapSettings {
            // This is the default, but we're setting it explicitly here for clarity.
            collision_object_names: ObjectNames::All,
            // Not related to current example, but center the map
            map_positioning: MapPositioning::Centered,
            ..default()
        },
        ..Default::default()
    });
}
