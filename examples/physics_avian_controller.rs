//! This example shows a simple player-controlled object using Avian2D physics. You can move the object using arrow keys.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

const MOVE_SPEED: f32 = 200.;
const GRAVITY_SCALE: f32 = 10.0;

// A 'player' marker component
#[derive(Default, Clone, Component)]
pub struct PlayerMarker;

// Define a custom physics collider which will use the TiledPhysicsAvianBackend
// but add an extra RigidBody::Static component on top of the colliders.
#[derive(Default, Clone, Reflect)]
struct MyCustomAvianPhysicsBackend(TiledPhysicsAvianBackend);

impl TiledPhysicsBackend for MyCustomAvianPhysicsBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        tiled_map: &TiledMap,
        filter: &TiledNameFilter,
        collider: &TiledCollider,
    ) -> Vec<TiledColliderSpawnInfos> {
        let colliders = self
            .0
            .spawn_colliders(commands, tiled_map, filter, collider);
        for c in &colliders {
            commands.entity(c.entity).insert(RigidBody::Static);
        }
        colliders
    }
}

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
        .add_systems(Update, move_player)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Text(
        "Move the ball using arrow keys or try to rotate the map!".to_string(),
    ));
    commands
        .spawn((
            TiledMapHandle(asset_server.load("maps/orthogonal/multiple_layers_with_colliders.tmx")),
            TiledMapSettings {
                layer_positioning: LayerPositioning::Centered,
                ..Default::default()
            },
        ))
        .observe(|_: Trigger<TiledMapCreated>, mut commands: Commands| {
            // Spawn a simple player-controlled object
            commands.spawn((
                RigidBody::Dynamic,
                PlayerMarker,
                Name::new("PlayerControlledObject (Avian2D physics)"),
                Collider::circle(10.),
                GravityScale(GRAVITY_SCALE),
                Transform::default(),
            ));
        });
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut LinearVelocity, With<PlayerMarker>>,
) {
    for mut rb_vel in player.iter_mut() {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction += Vec2::new(1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction -= Vec2::new(1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction += Vec2::new(0.0, 1.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction -= Vec2::new(0.0, 1.0);
        }

        if direction != Vec2::ZERO {
            direction /= direction.length();
        }

        rb_vel.0 = direction * MOVE_SPEED;
    }
}
