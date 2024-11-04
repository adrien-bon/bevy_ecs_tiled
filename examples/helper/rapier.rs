use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

const MOVE_SPEED: f32 = 200.;
const GRAVITY_SCALE: f32 = 10.0;

#[derive(Default, Clone, Component)]
pub struct PlayerMarker;

#[allow(unused)]
pub fn spawn_player(commands: &mut Commands, radius: f32, spawn_position: Vec2) {
    commands
        .spawn(RigidBody::Dynamic)
        .insert(PlayerMarker)
        .insert(Name::new("PlayerControlledObject (Rapier physics)"))
        .insert(Collider::ball(radius))
        .insert(Velocity::zero())
        .insert(GravityScale(GRAVITY_SCALE))
        .insert(Transform::from_xyz(spawn_position.x, spawn_position.y, 0.0));
}

pub fn move_player(
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
