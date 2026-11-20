//! Make an entity walk back and forth along the vertices of a Tiled object.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::{UpdateSystems, controller::MovementIntent};

/// How close to a waypoint, in pixels, we need to be to consider it reached.
const WAYPOINT_REACHED_DISTANCE: f32 = 10.;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PatrolRoute>();
    app.add_systems(
        Update,
        (select_next_waypoint, move_to_waypoint)
            .chain()
            .in_set(UpdateSystems::RecordInput),
    );
}

/// The Tiled object to patrol along: we walk from one of its vertices to the next one.
///
/// It is added from Tiled, where it references a polyline object.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[require(PatrolProgress)]
#[reflect(Component)]
pub struct PatrolRoute(pub Entity);

/// Where we are along our [`PatrolRoute`].
///
/// This is a runtime state, not something to set from Tiled: it does not derive
/// `Reflect`, so it is not part of the types we export to Tiled.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct PatrolProgress {
    /// Index of the next vertex of the route to select.
    pub next_waypoint: usize,
    /// Horizontal position we are currently walking to, if any.
    pub destination: Option<f32>,
}

/// Once the current waypoint has been reached, select the next one.
fn select_next_waypoint(
    maps_assets: Res<Assets<TiledMapAsset>>,
    map_query: Query<&TiledMap>,
    mut patrolling_actor_query: Query<(&PatrolRoute, &mut PatrolProgress)>,
    patrol_route_query: Query<(&TiledObject, &TiledMapReference, &GlobalTransform)>,
) {
    for (route, mut progress) in patrolling_actor_query.iter_mut() {
        if progress.destination.is_some() {
            continue;
        }

        // Retrieve the vertices of the Tiled object, in world space
        let Some(vertices) = patrol_route_query.get(route.0).ok().and_then(
            |(route, map_reference, route_transform)| {
                map_query
                    .get(map_reference.entity())
                    .ok()
                    .and_then(|map_handle| maps_assets.get(&map_handle.0))
                    .map(|map_asset| map_asset.object_vertices(route, route_transform))
            },
        ) else {
            continue;
        };

        let index = progress.next_waypoint % vertices.len().max(1);
        let Some(target) = vertices.get(index) else {
            continue;
        };
        progress.destination = Some(target.x);
        progress.next_waypoint = index + 1;
    }
}

/// Walk toward the current waypoint.
fn move_to_waypoint(
    mut patrolling_actor_query: Query<(&GlobalTransform, &mut PatrolProgress, &mut MovementIntent)>,
) {
    for (transform, mut progress, mut intent) in patrolling_actor_query.iter_mut() {
        let Some(destination) = progress.destination else {
            intent.direction = 0.;
            continue;
        };

        let distance = destination - transform.translation().x;
        if distance.abs() < WAYPOINT_REACHED_DISTANCE {
            progress.destination = None;
            intent.direction = 0.;
        } else {
            intent.direction = distance.signum();
        }
    }
}
