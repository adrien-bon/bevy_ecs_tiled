//! One-way platforms: actors can jump through them from below and land on top of them.
//!
//! On the Tiled side, platforms are regular tiles with a collision shape, drawn on a dedicated
//! tiles layer which has the [`OneWayPlatform`] user property. `bevy_ecs_tiled` physics creates
//! the colliders for us and we only have to tag them with [`OneWayPlatformCollider`].
//!
//! On the physics side, we rely on Avian collision hooks to ignore contacts with these
//! colliders when an actor should pass through them. This part is adapted from Avian examples:
//! <https://github.com/avianphysics/avian/blob/main/crates/avian2d/examples/one_way_platform_2d.rs>

use avian2d::prelude::*;
use bevy::{
    ecs::{
        entity::hash_set::EntityHashSet,
        system::{SystemParam, lifetimeless::Read},
    },
    prelude::*,
};
use bevy_ecs_tiled::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<OneWayPlatform>();
    app.register_type::<PassThroughOneWayPlatform>();
    app.add_observer(on_collider_created);
    app.add_systems(FixedUpdate, refresh_contacts_on_pass_through_change);
}

/// Marker component for a Tiled layer whose tiles are one-way platforms.
///
/// It is added from Tiled, on a tiles layer.
///
/// We require [`TiledColliderStrategy::LineStrip`] for this layer, overriding the strategy
/// of the map: we get a separate collider for each platform instead of a single one for
/// the whole layer, so each platform tracks on its own which actors are passing through it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(TiledColliderStrategy::LineStrip)]
#[reflect(Component, Default)]
pub struct OneWayPlatform;

/// A collider of a [`OneWayPlatform`], with the entities currently passing through it.
///
/// Contact modification is enabled for these colliders with the `ActiveCollisionHooks` component.
#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
#[require(ActiveCollisionHooks::MODIFY_CONTACTS)]
pub struct OneWayPlatformCollider(EntityHashSet);

/// A component to control how an actor interacts with a one-way platform.
///
/// We require [`SpeculativeCcd`] so contacts with a platform are detected slightly before
/// the actor actually touches it. Otherwise, the first contact of an actor landing on a platform
/// would already be a penetrating one, and it would be handled as if the actor was still
/// passing through the platform since its last jump from below.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(SpeculativeCcd)]
#[reflect(Component, Default)]
pub enum PassThroughOneWayPlatform {
    /// Passes through a one-way platform if the contact normal is in line with the platform's local-space up vector.
    #[default]
    ByNormal,
    /// Always passes through a one-way platform, temporarily set this to allow an actor to jump down through a platform.
    Always,
    /// Never passes through a one-way platform.
    Never,
}

/// Tag the colliders created for a [`OneWayPlatform`] layer.
///
/// Depending on the physics settings, a collider is either added to the layer entity itself
/// or to one of its descendants: the event always references the layer.
fn on_collider_created(
    collider_created: On<TiledEvent<ColliderCreated>>,
    mut commands: Commands,
    platform_query: Query<(), With<OneWayPlatform>>,
) {
    let event = collider_created.event();
    // Global observer: skip the copies of the event received while it bubbles up the hierarchy
    if !event.is_original() {
        return;
    }
    if event.event.source == TiledColliderSource::TilesLayer
        && event
            .layer_entity()
            .is_some_and(|layer| platform_query.contains(layer))
    {
        // As of Avian 0.8, `ActiveCollisionHooks::MODIFY_CONTACTS` is only taken into account
        // when a collider is attached to its rigid body, not when it is inserted afterward.
        // Since the collider already exists at this point, insert the rigid body
        // (again, if it was already there) along with our component to work around it.
        commands
            .entity(event.origin)
            .insert((OneWayPlatformCollider::default(), RigidBody::Static));
    }
}

/// Make sure our collision hooks are called again when a [`PassThroughOneWayPlatform`] changes.
///
/// Avian recycles the contacts of colliders which barely moved relative to each other,
/// without calling the collision hooks again. It means the contacts of an actor standing still
/// on a platform would not be evaluated again when it asks to drop through it: to prevent that,
/// discard the data used to recycle the contacts between this actor and one-way platforms.
fn refresh_contacts_on_pass_through_change(
    actor_query: Query<Entity, Changed<PassThroughOneWayPlatform>>,
    platform_query: Query<(), With<OneWayPlatformCollider>>,
    mut contact_graph: ResMut<ContactGraph>,
) {
    for actor in &actor_query {
        for contacts in contact_graph.iter_active_mut() {
            if (contacts.collider1 == actor && platform_query.contains(contacts.collider2))
                || (contacts.collider2 == actor && platform_query.contains(contacts.collider1))
            {
                contacts.recycling_cache = None;
            }
        }
    }
}

/// Custom [`CollisionHooks`], to be registered using `PhysicsPlugins::with_collision_hooks()`.
///
/// It can have read-only access to queries, resources, and other system parameters.
#[derive(SystemParam)]
pub struct PlatformerCollisionHooks<'w, 's> {
    one_way_platforms_query: Query<'w, 's, (Read<OneWayPlatformCollider>, Read<GlobalTransform>)>,
    pass_through_query: Query<'w, 's, Read<PassThroughOneWayPlatform>>,
}

impl CollisionHooks for PlatformerCollisionHooks<'_, '_> {
    /// Allows entities to pass through [`OneWayPlatformCollider`] entities.
    ///
    /// Passing through is achieved by removing the collisions between the platform
    /// and the other entity if the entity should pass through.
    /// If a [`PassThroughOneWayPlatform`] is present on the non-platform entity,
    /// the value of the component dictates the pass-through behaviour, otherwise
    /// [`PassThroughOneWayPlatform::ByNormal`] is used.
    ///
    /// Entities known to be passing through a platform are stored in its [`OneWayPlatformCollider`]:
    /// such an entity is allowed to continue to do so until it is no longer penetrating the platform,
    /// even if [`PassThroughOneWayPlatform`] has been set to disallow passing through in the meantime.
    fn modify_contacts(&self, contacts: &mut ContactPair, commands: &mut Commands) -> bool {
        // This is the contact modification hook, called after collision detection,
        // but before constraints are created for the solver. Mutable access to the ECS
        // is not allowed, but we can queue commands to perform deferred changes.

        // First, figure out which entity is the one-way platform, and which is the other.
        // Manifold normals point from the first collider to the second one: we want them
        // to point from the platform to the other entity.
        let (platform_entity, one_way_platform, platform_transform, other_entity, normal_sign) =
            if let Ok((one_way_platform, platform_transform)) =
                self.one_way_platforms_query.get(contacts.collider1)
            {
                (
                    contacts.collider1,
                    one_way_platform,
                    platform_transform,
                    contacts.collider2,
                    1.,
                )
            } else if let Ok((one_way_platform, platform_transform)) =
                self.one_way_platforms_query.get(contacts.collider2)
            {
                (
                    contacts.collider2,
                    one_way_platform,
                    platform_transform,
                    contacts.collider1,
                    -1.,
                )
            } else {
                // Neither is a one-way-platform, so accept the collision: we're done here.
                return true;
            };

        if one_way_platform.0.contains(&other_entity) {
            let any_penetrating = contacts.manifolds.iter().any(|manifold| {
                manifold
                    .points
                    .iter()
                    .any(|contact| contact.penetration > 0.0)
            });

            if any_penetrating {
                // If we were already allowing a collision for a particular entity,
                // and if it is penetrating us still, continue to allow it to do so.
                return false;
            } else {
                // If it's no longer penetrating us, forget it.
                commands.queue(OneWayPlatformCommand::Remove {
                    platform_entity,
                    entity: other_entity,
                });
            }
        }

        // Default behaviour is "by normal" for entities which do not have the component.
        match self
            .pass_through_query
            .get(other_entity)
            .copied()
            .unwrap_or_default()
        {
            // Pass-through is set to never, so accept the collision.
            PassThroughOneWayPlatform::Never => true,
            // Pass-through is set to always, so always ignore this collision
            // and register it as an entity that's currently penetrating.
            PassThroughOneWayPlatform::Always => {
                commands.queue(OneWayPlatformCommand::Add {
                    platform_entity,
                    entity: other_entity,
                });
                false
            }
            PassThroughOneWayPlatform::ByNormal => {
                // If all contact normals are in line with the local up vector of this platform,
                // then this collision should occur: the entity is on top of the platform.
                let platform_up = platform_transform.up().truncate();
                if contacts.manifolds.iter().all(|manifold| {
                    let normal = normal_sign * manifold.normal;
                    normal.length() > f32::EPSILON && normal.dot(platform_up) >= 0.5
                }) {
                    true
                } else {
                    // Otherwise, ignore the collision and register
                    // the other entity as one that's currently penetrating.
                    commands.queue(OneWayPlatformCommand::Add {
                        platform_entity,
                        entity: other_entity,
                    });
                    false
                }
            }
        }
    }
}

/// A command to add/remove entities to/from the set of entities
/// that are currently in contact with a one-way platform.
enum OneWayPlatformCommand {
    Add {
        platform_entity: Entity,
        entity: Entity,
    },
    Remove {
        platform_entity: Entity,
        entity: Entity,
    },
}

impl Command for OneWayPlatformCommand {
    type Out = ();

    fn apply(self, world: &mut World) {
        match self {
            OneWayPlatformCommand::Add {
                platform_entity,
                entity,
            } => {
                if let Some(mut platform) = world.get_mut::<OneWayPlatformCollider>(platform_entity)
                {
                    platform.0.insert(entity);
                }
            }
            OneWayPlatformCommand::Remove {
                platform_entity,
                entity,
            } => {
                if let Some(mut platform) = world.get_mut::<OneWayPlatformCollider>(platform_entity)
                {
                    platform.0.remove(&entity);
                }
            }
        }
    }
}
