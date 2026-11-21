//! Physics props: objects such as crates which fall and are pushed around by the actors.
//!
//! On the Tiled side, a prop is a regular tile object with the [`DynamicProp`] user property:
//! its collider is created by `bevy_ecs_tiled` physics from the collision shapes of its tile.
//! The only thing we have to do is to make it a dynamic rigid body.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::trigger::{ActorKilled, TriggerActor};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<DynamicProp>();
    app.add_observer(on_collider_created);
    app.add_observer(despawn_prop_when_killed);
}

/// Marker component for a Tiled object which is a dynamic physics body.
///
/// It is added from Tiled, on a tile object whose tile has a collision shape.
///
/// We require [`TiledColliderStrategy::Triangulation`] for this object, overriding the strategy
/// of the map: the default [`TiledColliderStrategy::Polyline`] only creates the outline of the shape,
/// which has no area and thus no mass, while a dynamic body needs a solid collider.
///
/// We also require a small [`CollisionMargin`], which rounds the corners of the prop collider.
/// The ground is split in several colliders (see `tiles_chunk_size` in the `level` module):
/// without it, the flat side of a crate sliding on the ground would catch on the first vertex
/// of the next collider ("ghost collision") and the crate would be stuck at the junction.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(
    TiledColliderStrategy::Triangulation,
    CollisionMargin(2.),
    // Props can be teleported or killed by trigger zones, like the player
    TriggerActor
)]
#[reflect(Component, Default)]
pub struct DynamicProp;

/// Make a [`DynamicProp`] a dynamic rigid body, once its collider is created.
///
/// We do not insert the rigid body before: a dynamic body without a collider has no mass.
/// It is not overwritten by the static rigid body inserted on the other colliders of the map
/// (see the `level` module).
fn on_collider_created(
    collider_created: On<TiledEvent<ColliderCreated>>,
    mut commands: Commands,
    prop_query: Query<(), With<DynamicProp>>,
) {
    // Global observer: skip the copies of the event received while it bubbles up the hierarchy
    if !collider_created.is_original() {
        return;
    }
    let prop = collider_created.source_entity();
    if prop_query.contains(prop) {
        commands.entity(prop).insert(RigidBody::Dynamic);
    }
}

/// Remove a prop which fell in a kill zone.
fn despawn_prop_when_killed(
    actor_killed: On<ActorKilled>,
    mut commands: Commands,
    prop_query: Query<(), With<DynamicProp>>,
) {
    if prop_query.contains(actor_killed.entity) {
        commands.entity(actor_killed.entity).despawn();
    }
}
