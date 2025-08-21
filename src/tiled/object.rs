//! ECS components for Tiled objects.
//!
//! This module defines Bevy components used to represent Tiled objects within the ECS world.

use crate::prelude::*;
use crate::tiled::helpers::iso_projection;
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

    /// Apply rotation and scaling to given coordinate.
    fn apply_rotation_and_scaling(vertex: Vec2, transform: &GlobalTransform) -> Vec2 {
        let rotation = transform.rotation().to_euler(EulerRot::ZYX).0;
        let (cos, sin) = (rotation.cos(), rotation.sin());
        Vec2 {
            x: (vertex.x * cos - vertex.y * sin) * transform.scale().x,
            y: (vertex.x * sin + vertex.y * cos) * transform.scale().y,
        }
    }

    /// Returns the center position of the object in world space.
    ///
    /// The center is computed from the object's vertices, taking into account its shape and transformation.
    ///
    /// # Arguments
    /// * `transform` - The global transform to apply to the object.
    /// * `isometric_projection` - Wheter or not to perform an isometric projection.
    /// * `tilemap_size` - Size of the tilemap in tiles.
    /// * `tile_size` - Size of the tiles (or the grid cell) in pixels.
    /// * `offset` - Global map offset to apply.
    ///
    /// # Returns
    /// * `Option<Coord<f32>>` - The computed center, or `None` if not applicable.
    pub fn center(
        &self,
        transform: &GlobalTransform,
        isometric_projection: bool,
        tilemap_size: &TilemapSize,
        tile_size: &TilemapTileSize,
        offset: Vec2,
    ) -> Option<Coord<f32>> {
        MultiPoint::from(self.vertices(
            transform,
            isometric_projection,
            tilemap_size,
            tile_size,
            offset,
        ))
        .centroid()
        .map(|p| Coord { x: p.x(), y: p.y() })
    }

    /// Returns the vertices of the object in world space.
    ///
    /// Vertices are calculated based on the object's shape and its transformation (translation, rotation, scale).
    /// For isometric maps, the vertices are projected through the isometric transformation.
    ///
    /// # Arguments
    /// * `transform` - The global transform to apply to the object.
    /// * `isometric_projection` - Wheter or not to perform an isometric projection.
    /// * `tilemap_size` - Size of the tilemap in tiles.
    /// * `tile_size` - Size of the tiles (or the grid cell) in pixels.
    /// * `offset` - Global map offset to apply.
    ///
    /// # Returns
    /// * `Vec<Coord<f32>>` - The transformed vertices.
    pub fn vertices(
        &self,
        transform: &GlobalTransform,
        isometric_projection: bool,
        tilemap_size: &TilemapSize,
        tile_size: &TilemapTileSize,
        offset: Vec2,
    ) -> Vec<Coord<f32>> {
        // Get object world position
        let object_world_pos = Coord {
            x: transform.translation().x,
            y: transform.translation().y,
        };

        // Generate shape vertices relative to origin
        match self {
            TiledObject::Point | TiledObject::Text => vec![Vec2::ZERO],
            TiledObject::Tile { width, height } => {
                vec![
                    Vec2::new(0., 0.),          // Bottom-left relative to object
                    Vec2::new(0., *height),     // Top-left
                    Vec2::new(*width, *height), // Top-right
                    Vec2::new(*width, 0.),      // Bottom-right
                ]
            }
            TiledObject::Rectangle { width, height } => {
                vec![
                    Vec2::new(0., 0.),           // Top-left relative to object
                    Vec2::new(*width, 0.),       // Top-right
                    Vec2::new(*width, -*height), // Bottom-right
                    Vec2::new(0., -*height),     // Bottom-left
                ]
            }
            TiledObject::Ellipse { width, height } => (0..Self::ELLIPSE_NUM_POINTS)
                .map(|i| {
                    let theta =
                        2.0 * std::f32::consts::PI * (i as f32) / (Self::ELLIPSE_NUM_POINTS as f32);
                    let local_x = width / 2.0 * theta.cos() + width / 2.0;
                    let local_y = height / 2.0 * theta.sin() - height / 2.0;
                    Vec2::new(local_x, local_y)
                })
                .collect(),
            TiledObject::Polyline { vertices } | TiledObject::Polygon { vertices } => {
                vertices.clone()
            }
        }
        .into_iter()
        .map(|v| Self::apply_rotation_and_scaling(v, transform))
        .map(|v| {
            if isometric_projection && !matches!(self, TiledObject::Tile { .. }) {
                let offset_projected = iso_projection(
                    Vec2::new(offset.x + v.x, offset.y - v.y),
                    tilemap_size,
                    tile_size,
                );
                let origin_projected = iso_projection(offset, tilemap_size, tile_size);
                let relative_projected = offset_projected - origin_projected;

                Coord {
                    x: object_world_pos.x + relative_projected.x,
                    y: object_world_pos.y - relative_projected.y,
                }
            } else {
                Coord {
                    x: v.x + object_world_pos.x,
                    y: v.y + object_world_pos.y,
                }
            }
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
    /// * `isometric_projection` - Wheter or not to perform an isometric projection.
    /// * `tilemap_size` - Size of the tilemap in tiles.
    /// * `tile_size` - Size of the tiles (or the grid cell) in pixels.
    /// * `offset` - Global map offset to apply.
    ///
    /// # Returns
    /// * `Option<LineString<f32>>` - The resulting line string, or `None` if not applicable.
    pub fn line_string(
        &self,
        transform: &GlobalTransform,
        isometric_projection: bool,
        tilemap_size: &TilemapSize,
        tile_size: &TilemapTileSize,
        offset: Vec2,
    ) -> Option<LineString<f32>> {
        let coords = self.vertices(
            transform,
            isometric_projection,
            tilemap_size,
            tile_size,
            offset,
        );
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
    /// * `isometric_projection` - Wheter or not to perform an isometric projection.
    /// * `tilemap_size` - Size of the tilemap in tiles.
    /// * `tile_size` - Size of the tiles (or the grid cell) in pixels.
    /// * `offset` - Global map offset to apply.
    ///
    /// # Returns
    /// * `Option<GeoPolygon<f32>>` - The resulting polygon, or `None` if not applicable.
    pub fn polygon(
        &self,
        transform: &GlobalTransform,
        isometric_projection: bool,
        tilemap_size: &TilemapSize,
        tile_size: &TilemapTileSize,
        offset: Vec2,
    ) -> Option<GeoPolygon<f32>> {
        self.line_string(
            transform,
            isometric_projection,
            tilemap_size,
            tile_size,
            offset,
        )
        .and_then(|ls| match ls.is_closed() {
            true => Some(GeoPolygon::new(ls, vec![])),
            false => None,
        })
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledObject>();
}
