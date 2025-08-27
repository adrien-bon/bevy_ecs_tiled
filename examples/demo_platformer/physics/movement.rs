use avian2d::prelude::*;
use bevy::prelude::*;

use crate::UpdateSystems;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, apply_movement.in_set(UpdateSystems::ApplyMovement));
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[require(
    //TransformInterpolation,
    LockedAxes::ROTATION_LOCKED,
    RigidBody::Dynamic,
    CollisionEventsEnabled,
)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
}

impl MovementController {
    pub fn from_max_speed(max_speed: f32) -> Self {
        Self {
            max_speed,
            ..default()
        }
    }
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            // 400 pixels per second is a nice default, but we can still vary this per character.
            max_speed: 400.0,
        }
    }
}

fn apply_movement(mut movement_query: Query<(&MovementController, &mut LinearVelocity)>) {
    for (controller, mut velocity) in &mut movement_query {
        velocity.0 = controller.max_speed * controller.intent;
    }
}
