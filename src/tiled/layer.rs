//! ECS components for Tiled layers.
//!
//! This module defines Bevy components used to represent Tiled layers within the ECS world.
//! The [`TiledLayer`] enum allows systems to identify and differentiate between various types of Tiled layers,
//! such as tile layers, object layers, image layers, and group layers.

use bevy::prelude::*;

/// Marker [`Component`] for a Tiled map layer.
///
/// This enum is attached to entities representing Tiled layers in the ECS world.
/// Each variant corresponds to a specific Tiled layer type and indicates the expected children entities.
///
/// - `Tiles`: A layer containing tiles, parent of [`TiledTilemap`](crate::tiled::tile::TiledTilemap) entities.
/// - `Objects`: A layer containing objects, parent of [`TiledObject`](crate::tiled::object::TiledObject) entities.
/// - `Image`: A layer containing an image, parent of a single [`TiledImage`](crate::tiled::image::TiledImage) entity.
/// - `Group`: A group of layers, used to organize multiple layers hierarchically.
/// - `Unknown`: Fallback for unrecognized or unsupported layer types.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub enum TiledLayer {
    /// Unknown layer type, used as a fallback when the layer type cannot be determined.
    #[default]
    Unknown,
    /// A layer containing tiles.
    ///
    /// Parent of [`TiledTilemap`](crate::tiled::tile::TiledTilemap) entities.
    Tiles,
    /// A layer containing objects.
    ///
    /// Parent of [`TiledObject`](crate::tiled::object::TiledObject) entities.
    Objects,
    /// A layer containing an image.
    ///
    /// Parent of a single [`TiledImage`](crate::tiled::image::TiledImage) entity.
    Image,
    /// A group of layers, used to organize multiple layers hierarchically.
    Group,
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledLayer>();
}
