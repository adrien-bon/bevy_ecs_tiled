use std::time::Duration;

use avian2d::{math::*, prelude::*};
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_tiled::prelude::*;

use crate::{
    animation::{Animation, AnimationState, AnimationStateConfig},
    controller::{CharacterControllerBundle, MovementAction, MovementEvent},
};

const ENEMY_SPRITE_FILE: &str =
    "demo_platformer/kenney_platformer-pack-redux/Spritesheets/spritesheet_enemies.png";

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Enemy>();
    app.register_type::<PatrolRoute>();
    app.register_type::<PatrolProgress>();
    app.add_observer(on_enemy_added);
    app.add_systems(Update, move_enemy_along_patrol_route);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(Transform, Visibility, PatrolProgress)]
#[reflect(Component)]
pub struct Enemy;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[require(Transform, Visibility)]
#[reflect(Component)]
pub struct PatrolRoute(pub Entity);

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
#[require(Transform, Visibility)]
#[reflect(Component, Default)]
pub struct PatrolProgress(pub usize);

fn on_enemy_added(
    trigger: Trigger<OnAdd, Enemy>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout =
        TextureAtlasLayout::from_grid(UVec2::splat(128), 8, 16, Some(UVec2::splat(2)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let enemy_animation = Animation::default()
        .add_config(
            AnimationState::Idling,
            AnimationStateConfig {
                duration: Duration::from_millis(100),
                frames: vec![75, 83],
            },
        )
        .add_config(
            AnimationState::Walking,
            AnimationStateConfig {
                duration: Duration::from_millis(100),
                frames: vec![75, 83],
            },
        );

    commands.entity(trigger.target()).insert((
        Name::new("Enemy"),
        Sprite {
            image: asset_server.load(ENEMY_SPRITE_FILE),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 75,
            }),
            anchor: Anchor::Custom(Vec2::new(0., -0.1)),
            ..Default::default()
        },
        enemy_animation,
        CharacterControllerBundle::new(Collider::capsule(40., 30.)).with_movement(
            2000.,
            0.9,
            600.,
            PI * 0.45,
        ),
    ));
}

fn move_enemy_along_patrol_route(
    mut movement_event_writer: EventWriter<MovementEvent>,
    mut patrolling_enemy_query: Query<
        (Entity, &GlobalTransform, &PatrolRoute, &mut PatrolProgress),
        With<Enemy>,
    >,
    maps_assets: Res<Assets<TiledMapAsset>>,
    map_query: Query<&TiledMap>,
    objects_query: Query<(&TiledObject, &TiledMapReference, &GlobalTransform)>,
) {
    for (enemy, transform, route, mut progress) in patrolling_enemy_query.iter_mut() {
        let Some(vertices) =
            objects_query
                .get(route.0)
                .ok()
                .and_then(|(route, map_reference, route_transform)| {
                    map_query
                        .get(map_reference.0)
                        .ok()
                        .and_then(|map_handle| maps_assets.get(&map_handle.0))
                        .map(|map_asset| map_asset.object_vertices(route, route_transform))
                })
        else {
            continue;
        };

        if progress.0 >= vertices.len() {
            progress.0 = 0;
        }
        let target = vertices.get(progress.0).unwrap();
        let distance = target.x - transform.translation().x;
        info!("{}: target = {:?}, position = {}", enemy, target, distance);
        movement_event_writer.write(MovementEvent {
            entity: enemy,
            action: MovementAction::Move(if distance > 0. { 1. } else { -1. }),
        });

        if distance.abs() < 10. {
            progress.0 += 1;
        }
    }
}
