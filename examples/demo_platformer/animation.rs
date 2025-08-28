use bevy::{platform::collections::HashMap, prelude::*};
use std::time::Duration;

use crate::{physics::movement::MovementController, UpdateSystems};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.register_type::<Animation>();
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(UpdateSystems::TickTimers),
            (update_animation_movement, update_animation_atlas)
                .chain()
                .in_set(UpdateSystems::Update),
        ),
    );
}

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_movement(
    mut player_query: Query<(&MovementController, &mut Sprite, &mut Animation)>,
) {
    for (controller, mut sprite, mut animation) in &mut player_query {
        let dx = controller.intent.x;
        if dx != 0.0 {
            sprite.flip_x = dx < 0.0;
        }

        let animation_state = if controller.intent == Vec2::ZERO {
            AnimationState::Idling
        } else {
            AnimationState::Walking
        };
        animation.update_state(animation_state);
    }
}

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut Animation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(mut query: Query<(&Animation, &mut Sprite)>) {
    for (animation, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.changed() {
            atlas.index = animation.get_atlas_index();
        }
    }
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Animation {
    timer: Timer,
    frame_index: usize,
    state: AnimationState,
    #[reflect(ignore)]
    config: HashMap<AnimationState, AnimationStateConfig>,
}

pub struct AnimationStateConfig {
    duration: Duration,
    frames: Vec<usize>,
}

#[derive(Reflect, PartialEq)]
pub enum AnimationState {
    Idling,
    Walking,
}
