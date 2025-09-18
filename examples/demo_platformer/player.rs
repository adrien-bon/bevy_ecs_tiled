//! Player-specific behavior.

use std::time::Duration;

use crate::{
    animation::{Animation, AnimationState, AnimationStateConfig},
    controller::{CharacterControllerBundle, MovementAction, MovementEvent},
    UpdateSystems,
};
use avian2d::{math::*, prelude::*};
use bevy::{prelude::*, sprite::Anchor};

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

    let player_animation = Animation::default()
        .add_config(
            AnimationState::Idling,
            AnimationStateConfig {
                duration: Duration::from_millis(500),
                frames: vec![6],
            },
        )
        .add_config(
            AnimationState::Walking,
            AnimationStateConfig {
                duration: Duration::from_millis(100),
                frames: vec![37, 45],
            },
        );

    commands.spawn((
        Name::new("Player"),
        Player,
        spawn_transform,
        Sprite {
            image: asset_server.load(
                "demo_platformer/kenney_platformer-pack-redux/Spritesheets/spritesheet_players.png",
            ),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 6,
            }),
            anchor: Anchor::Custom(Vec2::new(0., -0.2)),
            ..Default::default()
        },
        player_animation,
        CharacterControllerBundle::new(Collider::capsule(50., 50.)).with_movement(
            5000.,
            0.9,
            600.,
            PI * 0.45,
        ),
    ));
}

/// Sends [`MovementAction`] events based on keyboard input.
fn record_player_directional_input(
    mut movement_event_writer: EventWriter<MovementEvent>,
    player_query: Query<Entity, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let horizontal = right as i8 - left as i8;
    let direction = horizontal as Scalar;

    if direction != 0.0 {
        movement_event_writer.write(MovementEvent {
            entity: player_entity,
            action: MovementAction::Move(direction),
        });
    }

    if keyboard_input.any_pressed([KeyCode::Space, KeyCode::KeyW, KeyCode::ArrowUp]) {
        movement_event_writer.write(MovementEvent {
            entity: player_entity,
            action: MovementAction::Jump,
        });
    }
}
