//! Player-specific behavior.

use crate::{physics::movement::MovementController, UpdateSystems};
use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.register_type::<PlayerSpawnPoint>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        record_player_directional_input.in_set(UpdateSystems::RecordInput),
    );
    app.add_observer(spawn_player_at_spawn_point);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(Transform, Visibility)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(Transform)]
#[reflect(Component)]
pub struct PlayerSpawnPoint;

fn spawn_player_at_spawn_point(
    t: Trigger<OnAdd, PlayerSpawnPoint>,
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    player_spawn_query: Query<&Transform, With<PlayerSpawnPoint>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if !player_query.is_empty() {
        // Player already exists, do nothing.
        return;
    }

    let spawn_transform = *player_spawn_query.get(t.target()).unwrap();

    let layout = TextureAtlasLayout::from_grid(UVec2::new(128, 256), 8, 8, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    //    let player_animation = PlayerAnimation::new();

    commands.spawn((
        Name::new("Player"),
        Player,
        spawn_transform,
        Sprite::from_atlas_image(
            asset_server.load(
                "demo_platformer/kenney_platformer-pack-redux/Spritesheets/spritesheet_players.png",
            ),
            TextureAtlas {
                layout: texture_atlas_layout,
                //                index: player_animation.get_atlas_index(),
                index: 6,
            },
        ),
        MovementController::from_max_speed(400.),
        Collider::rectangle(128., 256.),
        RigidBody::Dynamic,
    ));
}

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
    // This should be omitted if the input comes from an analog stick instead.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.intent = intent;
    }
}
