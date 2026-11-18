//! ECS components for Tiled images.
//!
//! This module defines Bevy components used to represent Tiled images within the ECS world.

use crate::prelude::*;
use bevy::prelude::*;

/// [`Component`] holding the image of an image layer.
///
/// The associated [`Transform`] is derived from this component: it is automatically updated when
/// this component changes, during [`TiledPostUpdateSystems::SyncRender`].
///
/// With the `render` feature, it is also the case of the associated [`Sprite`], and the
/// [`Transform`] of a repeated image is updated along its repeated axes to cover what the camera can see.
///
/// Without the `render` feature, the image is not loaded and this component has no image handle.
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub struct TiledImage {
    /// Image handle
    ///
    /// Only available with the `render` feature.
    #[cfg(feature = "render")]
    pub image: Handle<Image>,
    /// Base position, relative to parent layer
    pub base_position: Vec2,
    /// Base image size
    pub base_size: Vec2,
    /// Whether the image should be repeated along the X axis
    pub repeat_x: bool,
    /// Whether the image should be repeated along the Y axis
    pub repeat_y: bool,
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledImage>();
    app.add_systems(
        PostUpdate,
        sync_image_transform.in_set(TiledPostUpdateSystems::SyncRender),
    );
}

/// Update the [`Transform`] of new or modified [`TiledImage`]s
pub(crate) fn sync_image_transform(
    mut image_query: Query<(&TiledImage, &mut Transform), Changed<TiledImage>>,
) {
    for (image, mut transform) in image_query.iter_mut() {
        *transform = Transform::from_translation(image.base_position.extend(0.));
    }
}
