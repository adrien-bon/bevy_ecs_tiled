use avian2d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
use std::time::Duration;

use crate::UpdateSystems;

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
    mut player_query: Query<(&LinearVelocity, &mut Sprite, &mut Animation)>,
) {
    for (linear_velocity, mut sprite, mut animation) in &mut player_query {
        let dx = linear_velocity.x;
        if dx != 0. {
            sprite.flip_x = dx < 0.0;
        }

        let animation_state = if ops::abs(dx) < 10. {
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
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct Animation {
    timer: Timer,
    frame_index: usize,
    state: AnimationState,
    #[reflect(ignore)]
    config: HashMap<AnimationState, AnimationStateConfig>,
}

#[derive(Default, Debug, Clone)]
pub struct AnimationStateConfig {
    pub duration: Duration,
    pub frames: Vec<usize>,
}

#[derive(Reflect, Hash, Eq, PartialEq, Default, Debug, Copy, Clone)]
pub enum AnimationState {
    #[default]
    Unknown,
    Idling,
    Walking,
}

impl Animation {
    pub fn add_config(&mut self, state: AnimationState, config: AnimationStateConfig) -> Self {
        self.config.insert(state, config);
        self.clone()
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.is_finished() {
            return;
        }
        if let Some(config) = self.config.get(&self.state) {
            self.frame_index = (self.frame_index + 1) % config.frames.len();
        }
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: AnimationState) {
        if self.state != state {
            if let Some(config) = self.config.get(&state) {
                self.state = state;
                self.frame_index = 0;
                self.timer = Timer::new(config.duration, TimerMode::Repeating);
            }
        }
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        self.timer.is_finished()
    }

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        *self
            .config
            .get(&self.state)
            .and_then(|c| c.frames.get(self.frame_index))
            .unwrap_or(&0)
    }
}
