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
#[require(Transform, Visibility)]
#[reflect(Component)]
pub struct Enemy;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[require(Transform, Visibility)]
#[reflect(Component)]
pub struct PatrolRoute(pub Entity);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[require(Transform, Visibility)]
#[reflect(Component)]
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
            5000.,
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
    map_query: Query<(&TiledMap, &TiledMapStorage)>,
    objects_query: Query<(&TiledObject, &GlobalTransform)>,
) {
    for (enemy, transform, route, mut progress) in patrolling_enemy_query.iter_mut() {
        let Some((tilemap_size, grid_size)) = map_query
            .iter()
            .find_map(|(handle, storage)| {
                maps_assets
                    .get(&handle.0)
                    .and_then(|m| storage.get_object(&m.map, route.0).map(|_| m))
            })
            .map(|m| (m.tilemap_size, grid_size_from_map(&m.map)))
        else {
            continue;
        };
        let Ok((route, route_transform)) = objects_query.get(route.0) else {
            continue;
        };

        let vertices = route.vertices(
            route_transform,
            false,
            &tilemap_size,
            &grid_size,
            Vec2::ZERO,
        );
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
