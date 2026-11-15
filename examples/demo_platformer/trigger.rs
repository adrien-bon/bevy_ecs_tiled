//! Zones defined in Tiled which do something when an actor enters them.

use avian2d::prelude::*;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<TriggerZone>();
    app.register_type::<TriggerActor>();
    app.add_observer(on_zone_entered);
}

/// What happens when a [`TriggerActor`] enters the collider of this entity.
///
/// It is added from Tiled, on an object: `bevy_ecs_tiled` physics takes care of adding
/// the collider and we only have to turn it into a sensor.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[require(CollisionEventsEnabled, Sensor)]
pub enum TriggerZone {
    /// Kill the actor: see [`ActorKilled`].
    Kill,
    /// Move the actor to the position of the provided entity.
    Teleport(Entity),
}

/// Marker component for entities which interact with a [`TriggerZone`].
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, Default)]
pub struct TriggerActor;

/// Event triggered when a [`TriggerActor`] enters a [`TriggerZone::Kill`].
#[derive(EntityEvent, Debug, Clone, Copy)]
pub struct ActorKilled {
    pub entity: Entity,
}

/// Move a physics body at the same position than another entity and stop its movement.
///
/// Collision events are triggered in the middle of the physics step, where the `Transform`
/// of a body is about to be overwritten by the result of the simulation: we update
/// the physics [`Position`] instead, which is then written back to the `Transform`.
pub fn teleport(
    position: &mut Position,
    linear_velocity: &mut LinearVelocity,
    destination: &GlobalTransform,
) {
    position.0 = destination.translation().truncate();
    linear_velocity.0 = Vec2::ZERO;
}

fn on_zone_entered(
    collision_start: On<CollisionStart>,
    mut commands: Commands,
    zone_query: Query<&TriggerZone>,
    mut actor_query: Query<(&mut Position, &mut LinearVelocity), With<TriggerActor>>,
    destination_query: Query<&GlobalTransform>,
) {
    // The event is triggered for both colliders: `collider1` is the one it is
    // triggered for, so we only handle the case where it is the zone.
    let Ok(zone) = zone_query.get(collision_start.collider1) else {
        return;
    };
    let actor = collision_start.collider2;
    let Ok((mut position, mut linear_velocity)) = actor_query.get_mut(actor) else {
        return;
    };

    match zone {
        TriggerZone::Kill => {
            commands.trigger(ActorKilled { entity: actor });
        }
        TriggerZone::Teleport(destination) => {
            if let Ok(destination) = destination_query.get(*destination) {
                teleport(&mut position, &mut linear_velocity, destination);
            }
        }
    }
}
