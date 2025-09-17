//! This module contains all world [`Asset`]s definition.

use crate::prelude::*;
use bevy::{math::bounding::Aabb2d, prelude::*};
use std::fmt;

/// Tiled world `Asset`.
///
/// `Asset` holding Tiled world informations.
#[derive(TypePath, Asset)]
pub struct TiledWorldAsset {
    /// The raw Tiled world data
    pub world: tiled::World,
    /// World bounding box, unanchored
    ///
    /// Minimum is set at `(0., 0.)`
    /// Maximum is set at `(world_size.x, world_size.y)`
    pub rect: Rect,
    /// List of all the maps contained in this world
    ///
    /// Contains both the [`TiledMapAsset`] handle and its associated [`Rect`] boundary
    /// as defined by the `.world` file.
    /// Note that the actual map boundaries are not taken into account for world chunking.
    pub maps: Vec<(Rect, Handle<TiledMapAsset>)>,
}

impl TiledWorldAsset {
    /// Offset that should be applied to world underlying maps to account for the [`TilemapAnchor`]
    pub(crate) fn offset(&self, anchor: &TilemapAnchor) -> Vec2 {
        let min = &self.rect.min;
        let max = &self.rect.max;
        match anchor {
            TilemapAnchor::None => Vec2::ZERO,
            TilemapAnchor::TopLeft => Vec2::new(-min.x, -max.y),
            TilemapAnchor::TopRight => Vec2::new(-max.x, -max.y),
            TilemapAnchor::TopCenter => Vec2::new(-(max.x + min.x) / 2.0, -max.y),
            TilemapAnchor::CenterRight => Vec2::new(-max.x, -(max.y + min.y) / 2.0),
            TilemapAnchor::CenterLeft => Vec2::new(-min.x, -(max.y + min.y) / 2.0),
            TilemapAnchor::BottomLeft => Vec2::new(-min.x, -min.y),
            TilemapAnchor::BottomRight => Vec2::new(-max.x, -min.y),
            TilemapAnchor::BottomCenter => Vec2::new(-(max.x + min.x) / 2.0, -min.y),
            TilemapAnchor::Center => Vec2::new(-(max.x + min.x) / 2.0, -(max.y + min.y) / 2.0),
            TilemapAnchor::Custom(v) => Vec2::new(
                (-0.5 - v.x) * (max.x - min.x) - min.x,
                (-0.5 - v.y) * (max.y - min.y) - min.y,
            ),
        }
    }

    /// Iterate over all maps from this world
    pub(crate) fn for_each_map<F>(
        &self,
        world_transform: &GlobalTransform,
        anchor: &TilemapAnchor,
        mut f: F,
    ) where
        F: FnMut(u32, Aabb2d),
    {
        let offset = self.offset(anchor);
        let offset = offset.extend(0.0);
        let (_, r, t) = world_transform
            .mul_transform(Transform::from_translation(offset))
            .to_scale_rotation_translation();
        let (axis, mut angle) = r.to_axis_angle();
        if axis.z < 0. {
            angle = -angle;
        }
        let world_isometry = Isometry2d::new(Vec2::new(t.x, t.y), Rot2::radians(angle));
        for (idx, (rect, _)) in self.maps.iter().enumerate() {
            let idx = idx as u32;
            f(
                idx,
                Aabb2d::from_point_cloud(
                    Isometry2d::IDENTITY,
                    &[
                        world_isometry.transform_point(Vec2::new(rect.min.x, rect.min.y)),
                        world_isometry.transform_point(Vec2::new(rect.min.x, rect.max.y)),
                        world_isometry.transform_point(Vec2::new(rect.max.x, rect.max.y)),
                        world_isometry.transform_point(Vec2::new(rect.max.x, rect.min.y)),
                    ],
                ),
            );
        }
    }
}

impl fmt::Debug for TiledWorldAsset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TiledWorld")
            .field("world.source", &self.world.source)
            .field("rect", &self.rect)
            .finish()
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_asset::<TiledWorldAsset>();
}
