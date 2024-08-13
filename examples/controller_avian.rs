//! This example shows a simple player-controlled object using Avian2D physics. You can move the object using arrow keys.

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
        .add_plugins(helper::HelperPlugin)
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vec2::NEG_Y * 1000.0))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<TiledMap> = asset_server.load("multiple_layers_with_colliders.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        tiled_settings: TiledMapSettings {
            // This is the default, but we're setting it explicitly here for clarity.
            collision_object_names: ObjectNames::All,
            // By default, colliders are added without associated RigidBody
            // for this example, let's add a RigidBody::Static
            // you can also add any physic related component using this mecanism
            collider_callback: |entity_commands| {
                entity_commands.insert(RigidBody::Static);
            },
            ..default()
        },
        ..Default::default()
    });

    // Spawn a simple player-controlled object
    helper::avian::spawn_player(&mut commands, 10., Vec2::new(100., 100.));
}
