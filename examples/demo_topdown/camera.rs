use bevy::prelude::*;

use crate::{UpdateSystems, player::Player};

/// How fast the camera catches up with the player: the higher, the snappier.
const CAMERA_DECAY_RATE: f32 = 4.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_camera);
    app.add_systems(Update, camera_follow_player.in_set(UpdateSystems::Display));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d));
    commands.spawn((
        Name::new("Help text"),
        Text::new("Click on the ground to walk\nClick on a crate to break it"),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            left: px(12),
            ..default()
        },
    ));
}

/// Smoothly move the camera toward the player.
fn camera_follow_player(
    time: Res<Time>,
    camera_single: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player_single: Single<&Transform, With<Player>>,
) {
    let mut camera_transform = camera_single.into_inner();
    // Only follow the player on the X and Y axes: keep the camera depth
    let target = player_single
        .translation
        .truncate()
        .extend(camera_transform.translation.z);
    camera_transform
        .translation
        .smooth_nudge(&target, CAMERA_DECAY_RATE, time.delta_secs());
}
