//! This example shows a simple player-controlled object using Rapier physics. You can move the object using arrow keys.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
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
        .add_plugins(TiledPhysicsPlugin::<MyCustomRapierPhysicsBackend>::default())
        // Rapier physics plugins
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(TiledMapHandle(
        asset_server.load("multiple_layers_with_colliders.tmx"),
    ));

    // Spawn a simple player-controlled object
    helper::rapier::spawn_player(&mut commands, 10., Vec2::new(100., 100.));
}

// Define a custom physics collider which will use the TiledPhysicsRapierBackend
// but add an extra RigidBody::KinematicPositionBased component on top of the colliders.
#[derive(Default)]
struct MyCustomRapierPhysicsBackend(TiledPhysicsRapierBackend);

impl TiledPhysicsBackend for MyCustomRapierPhysicsBackend {
    fn spawn_collider(
        &self,
        commands: &mut Commands,
        map: &Map,
        collider_source: &TiledColliderSource,
    ) -> Option<TiledColliderSpawnInfos> {
        let collider = self.0.spawn_collider(commands, map, collider_source);
        if let Some(c) = &collider {
            commands
                .entity(c.entity)
                .insert(RigidBody::KinematicPositionBased);
        }
        collider
    }
}
