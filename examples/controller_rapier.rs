//! This example shows a simple player-controlled object using Rapier physics.

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

    let map_handle: Handle<TiledMap> = asset_server.load("multiple_layers_with_colliders.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        tiled_settings: TiledMapSettings {
            // This is the default, but we're setting it explicitly here for clarity.
            collision_object_names: ObjectNames::All,
            // By default, colliders are added without associated RigidBody
            // for this example, let's add a RigidBody::KinematicPositionBased
            // you can also add any physic related component using this mecanism
            collider_callback: |entity_commands| {
                entity_commands.insert(RigidBody::KinematicPositionBased);
            },
            ..default()
        },
        ..Default::default()
    });

    // Spawn a simple player-controlled object
    helper::rapier::spawn_player(&mut commands, 10., Vec2::new(100., 100.));
}
