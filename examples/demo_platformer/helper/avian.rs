use avian2d::prelude::*;
use bevy::prelude::*;

const MOVE_SPEED: f32 = 200.;
const GRAVITY_SCALE: f32 = 10.0;

#[derive(Default, Clone, Component)]
pub struct PlayerMarker;

#[allow(unused)]
pub fn spawn_player(commands: &mut Commands, radius: f32, spawn_position: Vec2) {
    commands
        .spawn(RigidBody::Dynamic)
        .insert(PlayerMarker)
        .insert(Name::new("PlayerControlledObject (Avian2D physics)"))
        .insert(Collider::circle(radius))
        .insert(GravityScale(GRAVITY_SCALE))
        .insert(Transform::from_xyz(spawn_position.x, spawn_position.y, 0.0));
}

pub fn move_player(
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
