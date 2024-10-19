//! This example shows a simple player-controlled object using Avian2D physics. You can move the object using arrow keys.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::Map;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // Examples helper plugin: contains the controller logic for this example
        .add_plugins(helper::HelperPlugin)
        // bevy_ecs_tilemap and bevy_ecs_tiled main plugins
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin::default())
        // bevy_ecs_tiled physics plugin: this is where we select which physics backend to use
        .add_plugins(TiledPhysicsPlugin::<MyCustomAvianPhysicsBackend>::default())
        // Avian physics plugins
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .add_plugins(PhysicsDebugPlugin::default())
        // Add gravity for this example
        .insert_resource(Gravity(Vec2::NEG_Y * 1000.0))
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TiledMapHandle(
        asset_server.load("multiple_layers_with_colliders.tmx"),
    ));

    // Spawn a simple player-controlled object
    helper::avian::spawn_player(&mut commands, 10., Vec2::new(100., 100.));
}
// Define a custom physics collider which will use the TiledPhysicsAvianBackend
// but add an extra RigidBody::Static component on top of the colliders.
#[derive(Default)]
struct MyCustomAvianPhysicsBackend(TiledPhysicsAvianBackend);

impl TiledPhysicsBackend for MyCustomAvianPhysicsBackend {
    fn spawn_collider(
        &self,
        commands: &mut Commands,
        map: &Map,
        collider_source: &TiledColliderSource,
    ) -> Option<TiledColliderSpawnInfos> {
        let collider = self.0.spawn_collider(commands, map, collider_source);
        if let Some(c) = &collider {
            commands.entity(c.entity).insert(RigidBody::Static);
        }
        collider
    }
}
