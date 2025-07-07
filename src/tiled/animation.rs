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
    /// First index of the animation
    pub start: usize,
    /// First index after the animation
    pub end: usize,
    /// Timer firing every time we should update the frame
    pub timer: Timer,
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledAnimation>();
    app.add_systems(
        Update,
        animate_sprite.in_set(TiledUpdateSystems::AnimateSprite),
    );
}

fn animate_sprite(time: Res<Time>, mut sprite_query: Query<(&mut TiledAnimation, &mut Sprite)>) {
    for (mut animation, mut sprite) in sprite_query.iter_mut() {
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index += 1;
                if atlas.index >= animation.end {
                    atlas.index = animation.start;
                }
            }
        }
    }
}
