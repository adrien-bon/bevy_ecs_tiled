//! ECS components for Tiled images.
//!
//! This module defines Bevy components used to represent Tiled images within the ECS world.

use bevy::prelude::*;

/// Marker [`Component`] for the [`Sprite`] attached to an image layer.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform, Sprite)]
pub struct TiledImage {
    pub base_position: Vec2,
    pub base_size: Vec2,
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledImage>();
    app.add_systems(Update, update_image_dimension);
}

fn update_image_dimension(
    mut image_query: Query<(&TiledImage, &mut Transform, &mut Sprite)>,
    camera_query: Query<(&Projection, &GlobalTransform)>,
) {
    if image_query.is_empty() {
        return;
    }

    let visible_area = camera_query
        .iter()
        .fold(Rect::EMPTY, |acc, (projection, transform)| {
            let Projection::Orthographic(p) = projection else {
                return acc;
            };
            let pos = transform.compute_transform().translation;
            let pos = Vec2::new(pos.x, pos.y);
            acc.union(Rect {
                min: pos + p.area.min,
                max: pos + p.area.max,
            })
        });

    for (image, mut transform, mut sprite) in image_query.iter_mut() {
        let (repeat_x, repeat_y) = match sprite.image_mode {
            SpriteImageMode::Tiled { tile_x, tile_y, .. } => (tile_x, tile_y),
            _ => continue,
        };

        // X axis tiling
        let (x, width) = if repeat_x {
            let tile_w = image.base_size.x;
            let base_x = image.base_position.x;
            let min_x = visible_area.min.x;
            let max_x = visible_area.max.x;
            let n = ((base_x - min_x) / tile_w).ceil().max(0.) + 1.;
            (
                base_x - n * tile_w,
                (max_x - base_x).abs().max(visible_area.width()) + 2. * tile_w,
            )
        } else {
            (image.base_position.x, image.base_size.x)
        };

        // Y axis tiling
        let (y, height) = if repeat_y {
            let tile_h = image.base_size.y;
            let base_y = image.base_position.y;
            let min_y = visible_area.max.y;
            let max_y = visible_area.min.y;
            let n = ((min_y - base_y) / tile_h).ceil().max(0.) + 1.;
            (
                base_y + n * tile_h,
                (max_y - base_y).abs().max(visible_area.height()) + 2. * tile_h,
            )
        } else {
            (image.base_position.y, image.base_size.y)
        };

        transform.translation.x = x;
        transform.translation.y = y;
        sprite.custom_size = Some(Vec2::new(width, height));
    }
}
