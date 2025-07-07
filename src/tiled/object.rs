//! ECS components for Tiled objects.
//!
//! This module defines Bevy components used to represent Tiled objects within the ECS world.

use bevy::prelude::*;

/// Marker [`Component`] for a Tiled map object.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub struct TiledObject;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledObject>();
}
