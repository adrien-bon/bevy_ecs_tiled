//! This example shows a simple player-controlled object using Avian2D physics. You can move the object using arrow keys.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

const MOVE_SPEED: f32 = 200.;
const GRAVITY_SCALE: f32 = 10.0;

fn main() {
    App::new()
        // Bevy default plugins: prevent blur effect by changing default sampling
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done
        .add_plugins(TiledPlugin::default())
        // Examples helper plugins, such as the logic to pan and zoom the camera
        // This should not be used directly in your game (but you can always have a look)
        .add_plugins(helper::HelperPlugin)
        // bevy_ecs_tiled physics plugin: this is where we select which physics backend to use
        // Here we use the provided Avian backend to automatically spawn colliders
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default())
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
    commands.spawn(Text(String::from(
        "Move the ball using arrow keys or try to rotate the map!",
    )));
    commands
        .spawn((
            TiledMap(asset_server.load("maps/orthogonal/multiple_layers_with_colliders.tmx")),
            TilemapAnchor::Center,
        ))
        // Wait for map loading to complete and spawn a simple player-controlled object
        .observe(
            |_: Trigger<TiledEvent<MapCreated>>, mut commands: Commands| {
                commands.spawn((
                    RigidBody::Dynamic,
                    PlayerMarker,
                    Name::new("PlayerControlledObject (Avian2D physics)"),
                    Collider::circle(10.),
                    GravityScale(GRAVITY_SCALE),
                    Transform::from_xyz(0., -50., 0.),
                ));
            },
        )
        // Automatically insert a `RigidBody::Static` component on all the colliders entities from the map
        .observe(
            |trigger: Trigger<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                commands
                    .entity(trigger.event().origin)
                    .insert(RigidBody::Static);
            },
        );
}

// A 'player' marker component
#[derive(Default, Clone, Component)]
pub struct PlayerMarker;

// A simplistic controller
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
