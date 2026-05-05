//! Animation systems for Tiled sprites.
//!
//! This module implements logic for animating Tiled tiles and objects with frame-based animations
//! as defined in Tiled maps.

use crate::prelude::*;
use bevy::prelude::*;

/// This [`Component`] is used for animated objects.
/// We will automatically update the Sprite index every time the timer fires.
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform, Sprite)]
pub struct TiledAnimation {
    /// All animation frames as (atlas_index, duration_secs) pairs.
    pub frames: Vec<(usize, f32)>,
    /// Index into `frames` currently being displayed.
    pub current_frame: usize,
    /// Counts down the current frame's individual duration.
    pub timer: Timer,
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledAnimation>();
    app.add_systems(
        Update,
        animate_sprite.in_set(TiledUpdateSystems::AnimateSprite),
    );
}

/// Advance `anim` by `delta`. Returns the new atlas index if the frame changed.
fn step_animation(anim: &mut TiledAnimation, delta: std::time::Duration) -> Option<usize> {
    if anim.frames.is_empty() {
        return None;
    }
    anim.timer.tick(delta);
    if anim.timer.just_finished() {
        anim.current_frame = (anim.current_frame + 1) % anim.frames.len();
        let (atlas_index, duration) = anim.frames[anim.current_frame];
        anim.timer = Timer::from_seconds(duration, TimerMode::Once);
        Some(atlas_index)
    } else {
        None
    }
}

fn animate_sprite(time: Res<Time>, mut query: Query<(&mut TiledAnimation, &mut Sprite)>) {
    for (mut anim, mut sprite) in query.iter_mut() {
        if let Some(atlas_index) = step_animation(&mut anim, time.delta()) {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = atlas_index;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn make_anim(frames: Vec<(usize, f32)>) -> TiledAnimation {
        let first_duration = frames.first().map(|(_, d)| *d).unwrap_or(0.1);
        TiledAnimation {
            frames,
            current_frame: 0,
            timer: Timer::from_seconds(first_duration, TimerMode::Once),
        }
    }

    #[test]
    fn empty_frames_does_nothing() {
        let mut anim = TiledAnimation::default();
        assert!(step_animation(&mut anim, Duration::from_secs(1)).is_none());
        assert_eq!(anim.current_frame, 0);
    }

    #[test]
    fn does_not_advance_before_timer_expires() {
        let mut anim = make_anim(vec![(0, 0.5), (1, 0.5)]);
        let result = step_animation(&mut anim, Duration::from_millis(100));
        assert!(result.is_none());
        assert_eq!(anim.current_frame, 0);
    }

    #[test]
    fn advances_to_next_frame_on_expiry() {
        let mut anim = make_anim(vec![(0, 0.1), (5, 0.1)]);
        // Use 101ms to account for f32→Duration precision: 0.1f32 > 100ms exactly
        let result = step_animation(&mut anim, Duration::from_millis(101));
        assert_eq!(result, Some(5)); // non-consecutive index
        assert_eq!(anim.current_frame, 1);
    }

    #[test]
    fn wraps_from_last_frame_to_first() {
        let mut anim = make_anim(vec![(0, 0.1), (5, 0.1)]);
        step_animation(&mut anim, Duration::from_millis(101)); // → frame 1
        let result = step_animation(&mut anim, Duration::from_millis(101)); // → frame 0
        assert_eq!(result, Some(0));
        assert_eq!(anim.current_frame, 0);
    }

    #[test]
    fn uses_per_frame_duration() {
        // Frame 0: 100ms, Frame 1: 500ms
        let mut anim = make_anim(vec![(0, 0.1), (1, 0.5)]);
        step_animation(&mut anim, Duration::from_millis(101)); // → frame 1 (100ms timer)
                                                               // Now on frame 1 with a 500ms timer — 100ms should NOT advance it
        let result = step_animation(&mut anim, Duration::from_millis(100));
        assert!(result.is_none());
        assert_eq!(anim.current_frame, 1);
    }
}
