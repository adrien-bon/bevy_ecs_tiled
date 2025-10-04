use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::player::Player;

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
        Name::new("Camera"),
        Camera2d,
        MainCamera,
        TiledParallaxCamera,
        IsDefaultUiCamera,
    ));
}

fn camera_follow_player(
    camera_single: Single<&mut Transform, With<MainCamera>>,
    player_single: Single<&GlobalTransform, With<Player>>,
) {
    let mut camera_transform = camera_single.into_inner();
    camera_transform.translation = player_single.into_inner().translation();
}
