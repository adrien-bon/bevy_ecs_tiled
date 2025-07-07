//! ECS components for Tiled images.
//!
//! This module defines Bevy components used to represent Tiled images within the ECS world.

use bevy::prelude::*;

/// Marker [`Component`] for the [`Sprite`] attached to an image layer.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform, Sprite)]
pub struct TiledImage;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledImage>();
}
