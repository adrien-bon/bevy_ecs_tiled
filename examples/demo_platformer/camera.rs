use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::player::Player;

/// How fast the camera catches up with the player: the higher, the snappier.
const CAMERA_DECAY_RATE: f32 = 8.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_camera);
    app.add_systems(
        PostUpdate,
        camera_follow_player.before(TransformSystems::Propagate),
    );
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("MainCamera"),
        Camera2d,
        MainCamera,
        TiledParallaxCamera,
        IsDefaultUiCamera,
    ));
}

/// Smoothly move the camera toward the player.
fn camera_follow_player(
    time: Res<Time>,
    camera_single: Single<&mut Transform, (With<MainCamera>, Without<Player>)>,
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
