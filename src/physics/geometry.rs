//! # Physics Geometry Utilities
//!
//! This module provides utilities for converting and processing geometric shapes extracted from Tiled maps
//! into formats suitable for physics simulation. It bridges the gap between Tiled's geometry representation
//! and the requirements of physics backends (Rapier, Avian, etc.).
use bevy::prelude::*;

use crate::prelude::*;
use ::geo::{BooleanOps, Centroid, TriangulateDelaunay};

/// Geometry of a physics collider, given to [`TiledPhysicsBackend::colliders`].
///
/// Besides the polygons of the collider, it holds the geometry of its neighbors: the colliders of
/// the adjacent tiles which belong to another chunk of the same tiles layer (see [`TiledColliderChunkSize`]).
/// The edges of the polygons shared with these neighbors are inside the layer geometry: they are not part
/// of its [`outline`](Self::outline), so a collider made of segments does not have an internal segment
/// at each chunk junction, which bodies sliding along the layer would hit.
#[derive(Clone, Debug, PartialEq)]
pub struct TiledColliderGeometry {
    /// Polygons of the collider, relative to the Tiled object or tiles layer entity.
    pub polygons: geo::MultiPolygon<f32>,
    /// Polygons of the neighbor colliders, in the same referential than [`Self::polygons`].
    ///
    /// Empty for Tiled objects and for tiles layers which are not split in chunks.
    pub neighbors: geo::MultiPolygon<f32>,
}

impl TiledColliderGeometry {
    /// Create the geometry of a collider without any neighbor.
    pub fn new(polygons: geo::MultiPolygon<f32>) -> Self {
        Self {
            polygons,
            neighbors: geo::MultiPolygon::new(vec![]),
        }
    }

    /// Outline of the collider: the rings of its polygons, minus the parts they share with its neighbors.
    ///
    /// Without neighbors, it is made of closed line strings, one for each ring.
    /// Otherwise, rings crossed by a neighbor are split in open line strings.
    pub fn outline(&self) -> geo::MultiLineString<f32> {
        let rings = geo::MultiLineString::new(
            multi_polygon_as_line_strings(&self.polygons)
                .cloned()
                .collect(),
        );
        if self.neighbors.0.is_empty() || rings.0.is_empty() {
            return rings;
        }
        // Remove the parts of the rings which are inside the neighbors or on their boundary
        self.neighbors.clip(&rings, true)
    }
}

/// Description of a collider shape, independent of any physics engine.
///
/// See [`collider_shapes`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum TiledColliderShape {
    /// A set of segments.
    Polyline {
        /// Vertices of the segments.
        vertices: Vec<Vec2>,
        /// Indices of the two vertices of each segment.
        ///
        /// If [`None`], vertices form a single line strip: each vertex is linked to the next one.
        indices: Option<Vec<[u32; 2]>>,
    },
    /// A set of triangles, meant to be aggregated in a single compound shape.
    ///
    /// Each triangle is given as its three vertices, relative to its centroid, and this centroid.
    Triangles(Vec<([Vec2; 3], Vec2)>),
}

/// Converts a [`TiledColliderGeometry`] into the [`TiledColliderShape`]s matching the given [`TiledColliderStrategy`].
///
/// Each returned shape is meant to be turned into a collider by a [`TiledPhysicsBackend`]:
/// this is what the Avian and Rapier backends do.
///
/// - [`TiledColliderStrategy::Polyline`]: a single [`TiledColliderShape::Polyline`] with all the segments
///   of the geometry [outline](TiledColliderGeometry::outline).
/// - [`TiledColliderStrategy::Triangulation`]: a single [`TiledColliderShape::Triangles`], made from the geometry
///   polygons. Since it is a solid shape, it does not depend on the neighbors of the geometry.
/// - [`TiledColliderStrategy::LineStrip`]: a [`TiledColliderShape::Polyline`] for each line string of the geometry
///   [outline](TiledColliderGeometry::outline).
/// - [`TiledColliderStrategy::Disabled`]: no shape at all.
///
/// Nothing is returned for an empty geometry.
pub fn collider_shapes(
    strategy: TiledColliderStrategy,
    geometry: &TiledColliderGeometry,
) -> Vec<TiledColliderShape> {
    match strategy {
        TiledColliderStrategy::Polyline => {
            let mut vertices = vec![];
            let mut indices = vec![];
            for line_string in &geometry.outline() {
                for line in line_string.lines() {
                    let len = vertices.len() as u32;
                    vertices.push(Vec2::new(line.start.x, line.start.y));
                    vertices.push(Vec2::new(line.end.x, line.end.y));
                    indices.push([len, len + 1]);
                }
            }
            if vertices.is_empty() {
                return vec![];
            }
            vec![TiledColliderShape::Polyline {
                vertices,
                indices: Some(indices),
            }]
        }
        TiledColliderStrategy::Triangulation => {
            let triangles = multi_polygon_as_triangles(&geometry.polygons);
            if triangles.is_empty() {
                return vec![];
            }
            vec![TiledColliderShape::Triangles(triangles)]
        }
        TiledColliderStrategy::LineStrip => geometry
            .outline()
            .into_iter()
            .filter(|line_string| line_string.0.len() > 1)
            .map(|line_string| TiledColliderShape::Polyline {
                vertices: line_string
                    .coords()
                    .map(|coord| Vec2::new(coord.x, coord.y))
                    .collect(),
                indices: None,
            })
            .collect(),
        TiledColliderStrategy::Disabled => vec![],
    }
}

/// Converts a [`geo::MultiPolygon<f32>`] into a vector of triangles and their centroids.
///
/// Each triangle is represented as an array of three [`Vec2`] points, relative to its centroid,
/// and its centroid as a [`Vec2`].
/// This is useful for physics backends that require triangulated shapes.
///
/// If the geometry cannot be triangulated, a warning is logged and an empty vector is returned.
pub fn multi_polygon_as_triangles(
    multi_polygon: &geo::MultiPolygon<f32>,
) -> Vec<([Vec2; 3], Vec2)> {
    let triangles = match multi_polygon.constrained_triangulation(Default::default()) {
        Ok(triangles) => triangles,
        Err(error) => {
            warn!("Failed to triangulate collider geometry: {error:?}");
            return vec![];
        }
    };
    triangles
        .into_iter()
        .map(|tri| {
            let (c_x, c_y) = tri.centroid().0.x_y();
            let d = Vec2::new(c_x, c_y);
            let tri = tri.to_array().map(|p| Vec2::new(p.x, p.y)).map(|p| p - d);

            (tri, d)
        })
        .collect()
}

/// Iterates over all the rings of a [`geo::MultiPolygon<f32>`], as [`geo::LineString<f32>`].
///
/// This function yields all interior and exterior rings from the input geometry.
/// Useful for physics backends that operate on polylines or linestrips.
pub fn multi_polygon_as_line_strings(
    multi_polygon: &geo::MultiPolygon<f32>,
) -> impl Iterator<Item = &geo::LineString<f32>> {
    multi_polygon
        .iter()
        .flat_map(|p| p.interiors().iter().chain(std::iter::once(p.exterior())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::geo::{Euclidean, Length, Rect, coord};

    fn square(min: (f32, f32), max: (f32, f32)) -> geo::Polygon<f32> {
        Rect::new(coord! { x: min.0, y: min.1 }, coord! { x: max.0, y: max.1 }).to_polygon()
    }

    fn outline_length(geometry: &TiledColliderGeometry) -> f32 {
        geometry
            .outline()
            .iter()
            .map(|line_string| Euclidean.length(line_string))
            .sum()
    }

    /// Whether a segment of the outline lies on the vertical line `x`, between `y0` and `y1`
    fn has_vertical_segment(geometry: &TiledColliderGeometry, x: f32, y0: f32, y1: f32) -> bool {
        geometry.outline().iter().any(|line_string| {
            line_string.lines().any(|line| {
                (line.start.x - x).abs() < 1e-4
                    && (line.end.x - x).abs() < 1e-4
                    && line.start.y.min(line.end.y) < y1
                    && line.start.y.max(line.end.y) > y0
            })
        })
    }

    #[test]
    fn outline_without_neighbors() {
        let geometry =
            TiledColliderGeometry::new(geo::MultiPolygon::new(vec![square((0., 0.), (2., 1.))]));
        let outline = geometry.outline();
        assert_eq!(outline.0.len(), 1);
        assert!(outline.0[0].is_closed());
        assert!((outline_length(&geometry) - 6.).abs() < 1e-4);
    }

    #[test]
    fn outline_skips_edges_shared_with_neighbors() {
        let geometry = TiledColliderGeometry {
            polygons: geo::MultiPolygon::new(vec![square((0., 0.), (2., 1.))]),
            neighbors: geo::MultiPolygon::new(vec![square((2., 0.), (3., 1.))]),
        };
        // Right side is shared with the neighbor
        assert!(!has_vertical_segment(&geometry, 2., 0., 1.));
        assert!(has_vertical_segment(&geometry, 0., 0., 1.));
        assert!((outline_length(&geometry) - 5.).abs() < 1e-4);
        // Polyline: all the remaining segments in a single shape
        let shapes = collider_shapes(TiledColliderStrategy::Polyline, &geometry);
        assert_eq!(shapes.len(), 1);
    }

    #[test]
    fn outline_keeps_edges_partially_shared_with_neighbors() {
        // Neighbor only covers the bottom half of the right side
        let geometry = TiledColliderGeometry {
            polygons: geo::MultiPolygon::new(vec![square((0., 0.), (2., 2.))]),
            neighbors: geo::MultiPolygon::new(vec![square((2., 0.), (3., 1.))]),
        };
        assert!(!has_vertical_segment(&geometry, 2., 0., 1.));
        assert!(has_vertical_segment(&geometry, 2., 1., 2.));
        assert!((outline_length(&geometry) - 7.).abs() < 1e-4);
    }

    #[test]
    fn triangulation_ignores_neighbors() {
        let polygons = geo::MultiPolygon::new(vec![square((0., 0.), (2., 1.))]);
        let alone = collider_shapes(
            TiledColliderStrategy::Triangulation,
            &TiledColliderGeometry::new(polygons.clone()),
        );
        let with_neighbors = collider_shapes(
            TiledColliderStrategy::Triangulation,
            &TiledColliderGeometry {
                polygons,
                neighbors: geo::MultiPolygon::new(vec![square((2., 0.), (3., 1.))]),
            },
        );
        assert_eq!(alone, with_neighbors);
    }
}
