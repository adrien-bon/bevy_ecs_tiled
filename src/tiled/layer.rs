//! ECS components for Tiled layers.
//!
//! This module defines Bevy components used to represent Tiled layers within the ECS world.
//! The [`TiledLayer`] enum allows systems to identify and differentiate between various types of Tiled layers,
//! such as tile layers, object layers, image layers, and group layers.

use crate::prelude::*;
use bevy::prelude::*;

/// Marker [`Component`] for a Tiled map layer.
///
/// This enum is attached to entities representing Tiled layers in the ECS world.
/// Each variant corresponds to a specific Tiled layer type and indicates the expected children entities.
///
/// - `Tiles`: A layer containing tiles, parent of [`TiledTilemap`] entities.
/// - `Objects`: A layer containing objects, parent of [`TiledObject`] entities.
/// - `Image`: A layer containing an image, parent of a single [`TiledImage`] entity.
/// - `Group`: A group of layers, used to organize multiple layers hierarchically.
#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Debug)]
#[require(Visibility, Transform)]
pub enum TiledLayer {
    /// A layer containing tiles.
    ///
    /// Parent of [`TiledTilemap`] entities.
    Tiles,
    /// A layer containing objects.
    ///
    /// Parent of [`TiledObject`] entities.
    Objects,
    /// A layer containing an image.
    ///
    /// Parent of a single [`TiledImage`] entity.
    Image,
    /// A group of layers, used to organize multiple layers hierarchically.
    Group,
}

/// Component that stores parallax information for Tiled layers.
#[derive(Component, Reflect, Clone, Debug, Copy)]
#[reflect(Component, Debug)]
pub struct TiledLayerParallax {
    /// The horizontal parallax multiplier.
    pub parallax_x: f32,
    /// The vertical parallax multiplier.
    pub parallax_y: f32,
    /// The base position of the layer before parallax is applied.
    pub base_position: Vec2,
}

/// Component that marks the camera to use for parallax calculations.
#[derive(Component, Reflect, Clone, Debug, Copy)]
#[reflect(Component, Debug)]
pub struct TiledParallaxCamera;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledLayer>();
    app.register_type::<TiledLayerParallax>();
    app.register_type::<TiledParallaxCamera>();
    app.add_systems(
        Update,
        update_layer_parallax.in_set(TiledUpdateSystems::UpdateParallaxLayers),
    );
}

fn update_layer_parallax(
    camera_query: Query<&Transform, (With<TiledParallaxCamera>, Changed<Transform>)>,
    mut layer_query: Query<(&TiledLayerParallax, &mut Transform), Without<TiledParallaxCamera>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    for (parallax, mut transform) in layer_query.iter_mut() {
        let camera_position = Vec2::new(
            camera_transform.translation.x,
            camera_transform.translation.y,
        );
        let parallax_offset = Vec2::new(
            camera_position.x * (1.0 - parallax.parallax_x),
            camera_position.y * (1.0 - parallax.parallax_y),
        );

        transform.translation.x = parallax.base_position.x + parallax_offset.x;
        transform.translation.y = parallax.base_position.y + parallax_offset.y;
    }
}
