use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::{
    controller::{MovementAction, MovementEvent},
    UpdateSystems,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<RequestedDestination>();
    app.register_type::<PatrolRoute>();
    app.register_type::<PatrolProgress>();
    app.add_systems(
        Update,
        (move_along_patrol_route, move_to_destination)
            .chain()
            .in_set(UpdateSystems::RecordInput),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct RequestedDestination(pub f32);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[require(PatrolProgress)]
#[reflect(Component)]
pub struct PatrolRoute(pub Entity);

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component, Default)]
pub struct PatrolProgress(pub usize);

fn move_to_destination(
    mut commands: Commands,
    mut movement_event_writer: EventWriter<MovementEvent>,
    mut patrolling_actor_query: Query<(Entity, &GlobalTransform, &RequestedDestination)>,
) {
    for (enemy, transform, target) in patrolling_actor_query.iter_mut() {
        let distance = target.0 - transform.translation().x;
        if distance.abs() < 10. {
            commands.entity(enemy).remove::<RequestedDestination>();
        } else {
            movement_event_writer.write(MovementEvent {
                entity: enemy,
                action: MovementAction::Move(if distance > 0. { 1. } else { -1. }),
            });
        }
    }
}

fn move_along_patrol_route(
    mut commands: Commands,
    maps_assets: Res<Assets<TiledMapAsset>>,
    map_query: Query<&TiledMap>,
    mut patrolling_actor_query: Query<
        (Entity, &PatrolRoute, &mut PatrolProgress),
        Without<RequestedDestination>,
    >,
    patrol_route_query: Query<(&TiledObject, &TiledMapReference, &GlobalTransform)>,
) {
    for (enemy, route, mut progress) in patrolling_actor_query.iter_mut() {
        let Some(vertices) = patrol_route_query.get(route.0).ok().and_then(
            |(route, map_reference, route_transform)| {
                map_query
                    .get(map_reference.0)
                    .ok()
                    .and_then(|map_handle| maps_assets.get(&map_handle.0))
                    .map(|map_asset| map_asset.object_vertices(route, route_transform))
            },
        ) else {
            continue;
        };

        if progress.0 >= vertices.len() {
            progress.0 = 0;
        }

        let target = vertices.get(progress.0).unwrap();
        commands
            .entity(enemy)
            .insert(RequestedDestination(target.x));
        progress.0 += 1;
    }
}
