//! A basic dynamic character controller, shared by the player and the enemies.
//!
//! This character controller is adapted from Avian examples:
//! <https://github.com/avianphysics/avian/tree/main/crates/avian2d/examples/dynamic_character_2d>
//!
//! Inputs (keyboard for the player, patrol logic for the enemies) are recorded in the
//! [`MovementIntent`] component during `Update`, then applied in `FixedUpdate`, just before
//! the physics step: this way, movement does not depend upon the framerate.

use avian2d::{
    math::{Real as Scalar, *},
    prelude::*,
};
use bevy::{ecs::query::Has, prelude::*};
use std::f32::consts::PI;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementAcceleration>();
    app.register_type::<MovementDampingFactor>();
    app.register_type::<JumpImpulse>();
    app.register_type::<MaxSlopeAngle>();
    app.add_systems(
        FixedUpdate,
        (update_grounded, movement, apply_movement_damping).chain(),
    );
    app.insert_resource(Gravity(RVector::NEG_Y * 1000.));
}

/// A marker component indicating that an entity is using a character controller.
///
/// Movement parameters can be tuned by overriding the required components.
#[derive(Component, Default)]
#[require(
    RigidBody::Dynamic,
    LockedAxes::ROTATION_LOCKED,
    MovementIntent,
    MovementAcceleration,
    MovementDampingFactor,
    JumpImpulse,
    MaxSlopeAngle
)]
pub struct CharacterController;

/// What the character is trying to do.
///
/// This is the only thing the player input or the enemies logic have to update.
#[derive(Component, Default)]
pub struct MovementIntent {
    /// Horizontal direction: negative to go left, positive to go right, zero to stand still.
    pub direction: Scalar,
    /// Whether the character wants to jump: it is reset once taken into account.
    pub jump: bool,
}

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// The acceleration used for character movement.
#[derive(Component, Reflect)]
pub struct MovementAcceleration(pub Scalar);

impl Default for MovementAcceleration {
    fn default() -> Self {
        Self(2000.)
    }
}

/// The damping factor used for slowing down movement.
#[derive(Component, Reflect)]
pub struct MovementDampingFactor(pub Scalar);

impl Default for MovementDampingFactor {
    fn default() -> Self {
        Self(0.9)
    }
}

/// The strength of a jump.
#[derive(Component, Reflect)]
pub struct JumpImpulse(pub Scalar);

impl Default for JumpImpulse {
    fn default() -> Self {
        Self(800.)
    }
}

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Component, Reflect)]
pub struct MaxSlopeAngle(pub Scalar);

impl Default for MaxSlopeAngle {
    fn default() -> Self {
        Self(PI * 0.45)
    }
}

/// Returns the components of a character controller using the provided collider.
pub fn character_controller(collider: Collider) -> impl Bundle {
    // Detect the ground using a slightly smaller version of the collider
    let mut caster_shape = collider.clone();
    caster_shape.set_scale(RVector::ONE * 0.99, 10);

    (
        CharacterController,
        ShapeCaster::new(caster_shape, RVector::ZERO, 0.0, Dir2::NEG_Y).with_max_distance(10.0),
        collider,
    )
}

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    query: Query<(Entity, &ShapeHits, &Rotation, &MaxSlopeAngle, Has<Grounded>)>,
) {
    for (entity, hits, rotation, max_slope_angle, was_grounded) in &query {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits
            .iter()
            .any(|hit| (rotation * -hit.normal2).angle_to(RVector::Y).abs() <= max_slope_angle.0);

        // Only update the component when the status actually changes
        if is_grounded && !was_grounded {
            commands.entity(entity).insert(Grounded);
        } else if !is_grounded && was_grounded {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Moves character controllers according to their [`MovementIntent`].
fn movement(
    time: Res<Time>,
    mut controllers: Query<(
        &mut MovementIntent,
        &MovementAcceleration,
        &JumpImpulse,
        &mut LinearVelocity,
        Has<Grounded>,
    )>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs_f64().real();

    for (mut intent, movement_acceleration, jump_impulse, mut linear_velocity, is_grounded) in
        &mut controllers
    {
        linear_velocity.x += intent.direction * movement_acceleration.0 * delta_time;

        // A jump request is dropped if the character is not on the ground
        if std::mem::take(&mut intent.jump) && is_grounded {
            linear_velocity.y = jump_impulse.0;
        }
    }
}

/// Slows down movement in the X direction.
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= damping_factor.0;
    }
}
