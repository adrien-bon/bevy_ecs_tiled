//! ECS components for Tiled objects.
//!
//! This module defines Bevy components used to represent Tiled objects within the ECS world.

use crate::prelude::*;
use bevy::prelude::*;
use geo::Centroid;
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
    const ELLIPSE_NUM_POINTS: u32 = 20;

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
    /// The center is computed from the object's vertices, taking into account its shape and transformation.
    ///
    /// # Arguments
    /// * `transform` - The global transform to apply to the object.
    ///
    /// # Returns
    /// * `Option<Coord<f32>>` - The computed center, or `None` if not applicable.
    pub fn center(&self, transform: &GlobalTransform) -> Option<Coord<f32>> {
        MultiPoint::from(self.vertices(transform))
            .centroid()
            .map(|p| Coord { x: p.x(), y: p.y() })
    }

    /// Returns the vertices of the object in world space.
    ///
    /// Vertices are calculated based on the object's shape and its transformation (translation, rotation, scale).
    ///
    /// # Arguments
    /// * `transform` - The global transform to apply to the object.
    ///
    /// # Returns
    /// * `Vec<Coord<f32>>` - The transformed vertices.
    pub fn vertices(&self, transform: &GlobalTransform) -> Vec<Coord<f32>> {
        let origin = Coord {
            x: transform.translation().x,
            y: transform.translation().y,
        };
        match self {
            TiledObject::Point | TiledObject::Text => vec![origin],
            TiledObject::Ellipse { width, height } => {
                let (cos_rotation, sin_rotation) = TiledObject::rotation_cos_sin(transform);
                let center = Coord {
                    x: origin.x + width / 2.0 * cos_rotation + height / 2.0 * sin_rotation,
                    y: origin.y + width / 2.0 * sin_rotation - height / 2.0 * cos_rotation,
                };
                (0..Self::ELLIPSE_NUM_POINTS)
                    .map(|i| {
                        let theta = 2.0 * std::f32::consts::PI * (i as f32)
                            / (Self::ELLIPSE_NUM_POINTS as f32);
                        let x = width / 2. * theta.cos();
                        let y = height / 2. * theta.sin();
                        center
                            + Coord {
                                x: x * cos_rotation - y * sin_rotation,
                                y: x * sin_rotation + y * cos_rotation,
                            }
                    })
                    .collect()
            }
            TiledObject::Rectangle { width, height } => {
                let (cos_rotation, sin_rotation) = TiledObject::rotation_cos_sin(transform);
                vec![
                    origin,
                    origin
                        + Coord {
                            x: height * sin_rotation,
                            y: -height * cos_rotation,
                        },
                    origin
                        + Coord {
                            x: width * cos_rotation + height * sin_rotation,
                            y: width * sin_rotation - height * cos_rotation,
                        },
                    origin
                        + Coord {
                            x: width * cos_rotation,
                            y: width * sin_rotation,
                        },
                ]
            }
            TiledObject::Tile { width, height } => {
                let (cos_rotation, sin_rotation) = TiledObject::rotation_cos_sin(transform);
                vec![
                    origin,
                    origin
                        + Coord {
                            x: width * cos_rotation,
                            y: width * sin_rotation,
                        },
                    origin
                        + Coord {
                            x: width * cos_rotation - height * sin_rotation,
                            y: width * sin_rotation + height * cos_rotation,
                        },
                    origin
                        + Coord {
                            x: -height * sin_rotation,
                            y: height * cos_rotation,
                        },
                ]
            }
            TiledObject::Polygon { vertices } | TiledObject::Polyline { vertices } => {
                let (cos_rotation, sin_rotation) = TiledObject::rotation_cos_sin(transform);
                vertices
                    .iter()
                    .map(|v| {
                        origin
                            + Coord {
                                x: v.x * cos_rotation - v.y * sin_rotation,
                                y: v.x * sin_rotation + v.y * cos_rotation,
                            }
                    })
                    .collect()
            }
        }
        .iter()
        .map(|c| Coord {
            x: (c.x - origin.x) * transform.scale().x + origin.x,
            y: (c.y - origin.y) * transform.scale().y + origin.y,
        })
        .collect()
    }

    /// Creates a [`LineString`] from the object's vertices.
    ///
    /// Returns `None` for point and text objects.
    /// For ellipses, rectangles, tiles, and polygons, returns a closed line string.
    /// For polylines, returns an open line string.
    ///
    /// # Arguments
    /// * `transform` - The global transform to apply to the object.
    ///
    /// # Returns
    /// * `Option<LineString<f32>>` - The resulting line string, or `None` if not applicable.
    pub fn line_string(&self, transform: &GlobalTransform) -> Option<LineString<f32>> {
        let coords = self.vertices(transform);
        match self {
            TiledObject::Point | TiledObject::Text => None,
            TiledObject::Ellipse { .. }
            | TiledObject::Rectangle { .. }
            | TiledObject::Tile { .. }
            | TiledObject::Polygon { .. } => {
                let mut line_string = LineString::from(coords);
                line_string.close();
                Some(line_string)
            }
            TiledObject::Polyline { .. } => Some(LineString::new(coords)),
        }
    }

    /// Creates a [`GeoPolygon`] from the object's vertices.
    ///
    /// Returns `None` for polyline, point, and text objects.
    /// For closed shapes, returns the corresponding polygon.
    ///
    /// # Arguments
    /// * `transform` - The global transform to apply to the object.
    ///
    /// # Returns
    /// * `Option<GeoPolygon<f32>>` - The resulting polygon, or `None` if not applicable.
    pub fn polygon(&self, transform: &GlobalTransform) -> Option<GeoPolygon<f32>> {
        self.line_string(transform)
            .and_then(|ls| match ls.is_closed() {
                true => Some(GeoPolygon::new(ls, vec![])),
                false => None,
            })
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledObject>();
}
