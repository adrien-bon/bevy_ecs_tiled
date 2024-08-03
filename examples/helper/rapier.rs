use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Default, Clone, Component)]
pub struct PlayerMarker;

#[allow(unused)]
pub fn spawn_player(commands: &mut Commands, radius: f32, spawn_position: Vec2) {
    commands
        .spawn(RigidBody::KinematicPositionBased)
        .insert(PlayerMarker)
        .insert(Name::new("PlayerControlledObject (Rapier physics)"))
        .insert(Collider::ball(radius))
        .insert(KinematicCharacterController::default())
        .insert(Restitution::coefficient(0.7))
        .insert(TransformBundle::from(Transform::from_xyz(
            spawn_position.x,
            spawn_position.y,
            0.0,
        )));
}

pub fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<(Entity, &mut KinematicCharacterController), With<PlayerMarker>>,
) {
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

    for (_, mut controller) in player.iter_mut() {
        controller.translation = Some(direction * 5.);
    }
}
