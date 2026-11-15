//! Player-specific behavior.

use std::time::Duration;

use crate::{
    UpdateSystems,
    animation::{Animation, AnimationState},
    controller::{MovementAcceleration, MovementIntent, character_controller},
    minimap::minimap_marker,
    platform::PassThroughOneWayPlatform,
    trigger::{ActorKilled, TriggerActor, teleport},
};
use avian2d::prelude::*;
use bevy::{color::palettes::css::BLUE, prelude::*, sprite::Anchor};

const PLAYER_SPRITE_FILE: &str =
    "demo_platformer/kenney_platformer-pack-redux/Spritesheets/spritesheet_players.png";

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.register_type::<PlayerSpawnPoint>();
    app.init_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        record_player_directional_input.in_set(UpdateSystems::RecordInput),
    );
    app.add_observer(spawn_player_at_spawn_point);
    app.add_observer(respawn_player_when_killed);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(Transform, Visibility, CollisionEventsEnabled)]
#[reflect(Component)]
pub struct Player;

/// Where the player starts, and where it is sent back when killed.
///
/// It is added from Tiled, on a point object.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(Transform)]
#[reflect(Component)]
pub struct PlayerSpawnPoint;

/// Assets shared by all the player entities: they are loaded only once.
#[derive(Resource)]
struct PlayerAssets {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            image: world.resource::<AssetServer>().load(PLAYER_SPRITE_FILE),
            layout: world.resource_mut::<Assets<TextureAtlasLayout>>().add(
                TextureAtlasLayout::from_grid(UVec2::new(128, 256), 8, 8, None, None),
            ),
        }
    }
}

fn spawn_player_at_spawn_point(
    add_player_spawn: On<Add<PlayerSpawnPoint>>,
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    transform_helper: TransformHelper,
    player_assets: Res<PlayerAssets>,
) {
    if !player_query.is_empty() {
        // Player already exists, do nothing.
        return;
    }

    // The spawn point is a Tiled object, which is a child of its layer and of the map:
    // we need its global position since the player does not belong to the map.
    // `GlobalTransform` is not updated yet at this point, so compute it ourselves.
    let Ok(spawn_transform) =
        transform_helper.compute_global_transform(add_player_spawn.event().entity)
    else {
        return;
    };

    let player_animation = Animation::default()
        .with_state(AnimationState::Idling, Duration::from_millis(500), vec![6])
        .with_state(
            AnimationState::Walking,
            Duration::from_millis(100),
            vec![37, 45],
        );

    commands.spawn((
        Name::new("Player"),
        Player,
        TriggerActor,
        Transform::from_translation(spawn_transform.translation()),
        Sprite {
            image: player_assets.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: player_assets.layout.clone(),
                index: 6,
            }),
            ..Default::default()
        },
        Anchor::from(Vec2::new(0., -0.2)),
        player_animation,
        character_controller(Collider::capsule(50., 50.)),
        MovementAcceleration(5000.),
        // Jump through one-way platforms from below and land on them
        PassThroughOneWayPlatform::ByNormal,
        children![minimap_marker(BLUE)],
    ));
}

/// Send the player back to its spawn point when it is killed.
fn respawn_player_when_killed(
    actor_killed: On<ActorKilled>,
    mut player_query: Query<(&mut Position, &mut LinearVelocity), With<Player>>,
    player_spawn_query: Query<&GlobalTransform, With<PlayerSpawnPoint>>,
) {
    let Ok((mut position, mut linear_velocity)) = player_query.get_mut(actor_killed.entity) else {
        return;
    };
    let Some(spawn_transform) = player_spawn_query.iter().next() else {
        return;
    };
    info!("Player was killed: back to the spawn point");
    teleport(&mut position, &mut linear_velocity, spawn_transform);
}

/// Updates player [`MovementIntent`] based on keyboard input.
fn record_player_directional_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut MovementIntent, &mut PassThroughOneWayPlatform), With<Player>>,
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let jump = keyboard_input.any_pressed([KeyCode::Space, KeyCode::KeyW, KeyCode::ArrowUp]);

    for (mut intent, mut pass_through) in &mut player_query {
        intent.direction = (right as i8 - left as i8).into();

        if down && jump {
            // Holding down while jumping: drop through the one-way platform we are standing on
            pass_through.set_if_neq(PassThroughOneWayPlatform::Always);
        } else {
            pass_through.set_if_neq(PassThroughOneWayPlatform::ByNormal);
            // Do not overwrite a pending jump request: movement is applied at a fixed
            // timestep, which could not have run yet since the key was pressed.
            intent.jump |= jump;
        }
    }
}
