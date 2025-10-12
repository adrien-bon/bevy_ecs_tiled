//! This example shows a simple player-controlled object using Rapier physics. You can move the object using arrow keys.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_rapier2d::prelude::*;

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
        // Here we use a custom backend (see below)
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsRapierBackend>::default())
        // Rapier physics plugins
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
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
            |_: On<TiledEvent<MapCreated>>, mut commands: Commands| {
                commands.spawn((
                    RigidBody::Dynamic,
                    PlayerMarker,
                    Name::new("PlayerControlledObject (Rapier physics)"),
                    Collider::ball(10.),
                    Velocity::zero(),
                    GravityScale(GRAVITY_SCALE),
                    Transform::from_xyz(50., -50., 0.),
                ));
            },
        )
        // Automatically insert a `RigidBody::Static` component on all the map entities
        .observe(
            |collider_created: On<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                commands
                    .entity(*collider_created.event().event.collider_of)
                    .insert(RigidBody::Fixed);
            },
        );
}

// A 'player' marker component
#[derive(Default, Clone, Component)]
pub struct PlayerMarker;

// A simplistic controller
fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut Velocity, With<PlayerMarker>>,
) {
    for mut rb_vels in player.iter_mut() {
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

        rb_vels.linvel = direction * MOVE_SPEED;
    }
}
