//! ECS components for Tiled objects.
//!
//! This module defines Bevy components used to represent Tiled objects within the ECS world.

use bevy::prelude::*;
use tiled::{ObjectData, ObjectShape};

/// Marker [`Component`] for a Tiled map object.
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub enum TiledObject {
    /// A point shape.
    #[default]
    Point,
    /// A rectangle shape.
    ///
    /// Anchor is at the top-left corner of the rect.
    Rectangle {
        /// The width of the rectangle.
        width: f32,
        /// The height of the rectangle.
        height: f32,
    },
    /// An ellipse shape.
    ///
    /// Anchor is at the top-left corner of the ellipse.
    Ellipse {
        /// The width of the ellipse.
        width: f32,
        /// The height of the ellipse.
        height: f32,
    },
    /// A polygon shape.
    Polygon {
        /// The vertices of the polygon, relative to object center.
        vertices: Vec<Vec2>,
    },
    /// A polyline shape.
    Polyline {
        /// The vertices of the polyline, relative to object center.
        vertices: Vec<Vec2>,
    },
    /// A tile object, which is a reference to a tile in a tilemap.
    ///
    /// Anchor is at the bottom-left corner of the tile.
    /// These objects usually have an associated [`Sprite`] and eventually
    /// a [`crate::prelude::TiledAnimation`] component.
    Tile {
        /// The width of the tile.
        width: f32,
        /// The height of the tile.
        height: f32,
    },
    /// A text object, which contains text data.
    ///
    /// Not supported yet.
    Text,
}

impl TiledObject {
    /// Creates a new [`TiledObject`] from the provided [`ObjectData`].
    pub fn from_object_data(object_data: &ObjectData) -> Self {
        if object_data.tile_data().is_some() {
            if let ObjectShape::Rect { width, height } = object_data.shape {
                TiledObject::Tile { width, height }
            } else {
                warn!(
                    "Object with tile data should have a rectangle shape, but found {:?}",
                    object_data.shape
                );
                TiledObject::default()
            }
        } else {
            match object_data.shape.clone() {
                ObjectShape::Point { .. } => TiledObject::Point,
                ObjectShape::Rect { width, height } => TiledObject::Rectangle { width, height },
                ObjectShape::Ellipse { width, height } => TiledObject::Ellipse { width, height },
                ObjectShape::Polygon { points } => TiledObject::Polygon {
                    vertices: points.into_iter().map(|(x, y)| Vec2::new(x, -y)).collect(),
                },
                ObjectShape::Polyline { points } => TiledObject::Polyline {
                    vertices: points.into_iter().map(|(x, y)| Vec2::new(x, -y)).collect(),
                },
                ObjectShape::Text { .. } => {
                    log::warn!("Text objects are not supported yet");
                    TiledObject::Text
                }
            }
        }
    }

    /// Returns the rotation cosinus and sinus of the object [`GlobalTransform`].
    fn rotation_cos_sin(transform: &GlobalTransform) -> (f32, f32) {
        let rotation = transform.rotation().to_euler(EulerRot::ZYX).0;
        (rotation.cos(), rotation.sin())
    }

    /// Returns the center position of the object in world space.
    ///
    /// The center is calculated based on the object's shape and its transformation.
    /// For point, text, polygon, and polyline objects, it returns the origin.
    /// For rectangle, ellipse and tile objects, it calculates the center based on their width, height, and rotation.
    pub fn center(&self, transform: &GlobalTransform) -> Vec2 {
        let origin = Vec2::new(transform.translation().x, transform.translation().y);
        match self {
            TiledObject::Point
            | TiledObject::Text
            | TiledObject::Polygon { vertices: _ }
            | TiledObject::Polyline { vertices: _ } => origin,
            TiledObject::Rectangle { width, height } | TiledObject::Ellipse { width, height } => {
                let (cos_rotation, sin_rotation) = TiledObject::rotation_cos_sin(transform);
                Vec2::new(
                    origin.x + *width / 2.0 * cos_rotation + *height / 2.0 * sin_rotation,
                    origin.y + *width / 2.0 * sin_rotation - *height / 2.0 * cos_rotation,
                )
            }
            TiledObject::Tile { width, height } => {
                let (cos_rotation, sin_rotation) = TiledObject::rotation_cos_sin(transform);
                Vec2::new(
                    origin.x + *width / 2.0 * cos_rotation - *height / 2.0 * sin_rotation,
                    origin.y + *width / 2.0 * sin_rotation + *height / 2.0 * cos_rotation,
                )
            }
        }
    }

    /// Returns the vertices of the object in world space.
    ///
    /// The vertices are calculated based on the object's shape and its transformation.
    /// For point and text objects, it returns a single vertex at the origin.
    /// For ellipse, rectangle, and tile objects, it calculates the vertices based on their width, height, and rotation.
    /// For polygon and polyline objects, it transforms the vertices based on the object's position and rotation.
    pub fn vertices(&self, transform: &GlobalTransform) -> Vec<Vec2> {
        let origin = Vec2::new(transform.translation().x, transform.translation().y);
        match self {
            TiledObject::Point | TiledObject::Text => vec![origin],
            TiledObject::Ellipse {
                width: _,
                height: _,
            } => vec![self.center(transform)],
            TiledObject::Rectangle { width, height } | TiledObject::Tile { width, height } => {
                let center = self.center(transform);
                let (cos_rotation, sin_rotation) = TiledObject::rotation_cos_sin(transform);
                vec![
                    Vec2::new(
                        center.x - *width / 2.0 * cos_rotation + *height / 2.0 * sin_rotation,
                        center.y - *width / 2.0 * sin_rotation - *height / 2.0 * cos_rotation,
                    ),
                    Vec2::new(
                        center.x + *width / 2.0 * cos_rotation + *height / 2.0 * sin_rotation,
                        center.y + *width / 2.0 * sin_rotation - *height / 2.0 * cos_rotation,
                    ),
                    Vec2::new(
                        center.x + *width / 2.0 * cos_rotation - *height / 2.0 * sin_rotation,
                        center.y + *width / 2.0 * sin_rotation + *height / 2.0 * cos_rotation,
                    ),
                    Vec2::new(
                        center.x - *width / 2.0 * cos_rotation - *height / 2.0 * sin_rotation,
                        center.y - *width / 2.0 * sin_rotation + *height / 2.0 * cos_rotation,
                    ),
                ]
            }
            TiledObject::Polygon { vertices } | TiledObject::Polyline { vertices } => {
                let (cos_rotation, sin_rotation) = TiledObject::rotation_cos_sin(transform);
                vertices
                    .iter()
                    .map(|v| {
                        Vec2::new(
                            origin.x + v.x * cos_rotation - v.y * sin_rotation,
                            origin.y + v.x * sin_rotation + v.y * cos_rotation,
                        )
                    })
                    .collect()
            }
        }
    }

    /// Returns the isometry of the object in 2D space.
    pub fn isometry_2d(&self, transform: &GlobalTransform) -> Isometry2d {
        Isometry2d {
            translation: self.center(transform),
            rotation: transform.rotation().to_euler(EulerRot::ZYX).0.into(),
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledObject>();
}
