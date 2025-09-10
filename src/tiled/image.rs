//! ECS components for Tiled images.
//!
//! This module defines Bevy components used to represent Tiled images within the ECS world.

use crate::prelude::*;
use bevy::prelude::*;

/// Marker [`Component`] for the [`Sprite`] attached to an image layer.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform, Sprite)]
pub struct TiledImage {
    /// Base position, relative to parent layer
    pub base_position: Vec2,
    /// Base image size
    pub base_size: Vec2,
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledImage>();
    app.add_systems(
        Update,
        update_image_position_and_size.in_set(TiledUpdateSystems::UpdateTiledImagePositionAndSize),
    );
}

fn update_image_position_and_size(
    mut image_query: Query<(&TiledImage, &ChildOf, &mut Transform, &mut Sprite), With<TiledImage>>,
    map_query: Query<&TiledMapImageRepeatMargin, With<TiledMap>>,
    layer_query: Query<(&GlobalTransform, &ChildOf), (With<TiledLayer>, Without<TiledImage>)>,
    camera_query: Query<(&Projection, &GlobalTransform), With<Camera2d>>,
) {
    // Early exit in case we don't have any image
    if image_query.is_empty() {
        return;
    }

    // Compute a visible area using all Camera2d
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

    for (image, child_of, mut transform, mut sprite) in image_query.iter_mut() {
        let (repeat_x, repeat_y) = match sprite.image_mode {
            SpriteImageMode::Tiled { tile_x, tile_y, .. } => (tile_x, tile_y),
            _ => continue,
        };

        // Skip to next image if this one does not repeat
        if !repeat_x && !repeat_y {
            continue;
        }

        // Retrieve layer transform from layer entity and image repeat margin from map entity
        let Ok((layer_transform, repeat_margin)) = layer_query
            .get(child_of.parent())
            .and_then(|(t, c)| map_query.get(c.parent()).map(|m| (t, m)))
        else {
            continue;
        };

        // Compute image absolute base position, using layer GlobalTransform
        let base = image.base_position.extend(0.) + layer_transform.translation();

        // X axis tiling
        let (x, width) = if repeat_x {
            let tile_w = image.base_size.x;
            let min_x = visible_area.min.x;
            let max_x = visible_area.max.x;
            let n = ((base.x - min_x) / tile_w).ceil().max(0.) + repeat_margin.0 as f32;
            (
                base.x - n * tile_w,
                (max_x - base.x).abs().max(visible_area.width())
                    + (1. + 2. * repeat_margin.0 as f32) * tile_w,
            )
        } else {
            (base.x, image.base_size.x)
        };

        // Y axis tiling
        let (y, height) = if repeat_y {
            let tile_h = image.base_size.y;
            let min_y = visible_area.max.y;
            let max_y = visible_area.min.y;
            let n = ((min_y - base.y) / tile_h).ceil().max(0.) + repeat_margin.0 as f32;
            (
                base.y + n * tile_h,
                (max_y - base.y).abs().max(visible_area.height())
                    + (1. + 2. * repeat_margin.0 as f32) * tile_h,
            )
        } else {
            (base.y, image.base_size.y)
        };

        // Update Sprite relative Transform and size
        transform.translation = Vec3::new(
            x - layer_transform.translation().x,
            y - layer_transform.translation().y,
            0.,
        );
        sprite.custom_size = Some(Vec2::new(width, height));
    }
}
