//! # Physics Geometry Utilities
//!
//! This module provides utilities for converting and processing geometric shapes extracted from Tiled maps
//! into formats suitable for physics simulation. It bridges the gap between Tiled's geometry representation
//! and the requirements of physics backends (Rapier, Avian, etc.).
use bevy::prelude::*;
use std::collections::VecDeque;

use crate::prelude::{
    geo::{Centroid, TriangulateDelaunay},
    *,
};

/// Description of a collider shape, independent of any physics engine.
///
/// See [`collider_shapes`].
#[derive(Clone, Debug, PartialEq)]
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

/// Converts a [`geo::MultiPolygon<f32>`] into the [`TiledColliderShape`]s matching the given [`TiledColliderStrategy`].
///
/// Each returned shape is meant to be turned into a collider by a [`TiledPhysicsBackend`]:
/// this is what the Avian and Rapier backends do.
///
/// - [`TiledColliderStrategy::Polyline`]: a single [`TiledColliderShape::Polyline`] with all the segments of the geometry.
/// - [`TiledColliderStrategy::Triangulation`]: a single [`TiledColliderShape::Triangles`].
/// - [`TiledColliderStrategy::LineStrip`]: a [`TiledColliderShape::Polyline`] for each ring of the geometry.
///
/// Nothing is returned for an empty geometry.
pub fn collider_shapes(
    strategy: TiledColliderStrategy,
    geometry: &geo::MultiPolygon<f32>,
) -> Vec<TiledColliderShape> {
    match strategy {
        TiledColliderStrategy::Polyline => {
            let mut vertices = vec![];
            let mut indices = vec![];
            for line_string in multi_polygon_as_line_strings(geometry) {
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
            let triangles = multi_polygon_as_triangles(geometry);
            if triangles.is_empty() {
                return vec![];
            }
            vec![TiledColliderShape::Triangles(triangles)]
        }
        TiledColliderStrategy::LineStrip => multi_polygon_as_line_strings(geometry)
            .filter(|line_string| line_string.0.len() > 1)
            .map(|line_string| TiledColliderShape::Polyline {
                vertices: line_string
                    .coords()
                    .map(|coord| Vec2::new(coord.x, coord.y))
                    .collect(),
                indices: None,
            })
            .collect(),
    }
}

/// Converts a [`geo::MultiPolygon<f32>`] into a vector of triangles and their centroids.
///
/// Each triangle is represented as an array of three [`Vec2`] points, relative to its centroid,
/// and its centroid as a [`Vec2`].
/// This is useful for physics backends that require triangulated shapes.
///
/// # Arguments
/// * `multi_polygon` - The input geometry to triangulate.
///
/// # Returns
/// A vector of tuples: ([triangle_vertices; 3], centroid).
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
///
/// # Arguments
/// * `multi_polygon` - The input geometry to extract line strings from.
///
/// # Returns
/// An iterator over the [`geo::LineString<f32>`] representing all rings in the geometry.
pub fn multi_polygon_as_line_strings(
    multi_polygon: &geo::MultiPolygon<f32>,
) -> impl Iterator<Item = &geo::LineString<f32>> {
    multi_polygon
        .iter()
        .flat_map(|p| p.interiors().iter().chain(std::iter::once(p.exterior())))
}

/// Reduces a collection of items into a single item using a binary reduction function.
///
/// This function implements a **tree reduction algorithm**: it repeatedly pairs consecutive items
/// and applies a reduction function to combine them, until only one item remains. This approach
/// is efficient for aggregating geometric shapes, combining colliders, or reducing complex data.
///
/// # Arguments
/// * `list` - The collection of items to reduce. If empty, returns `None`.
/// * `reduction` - A closure that combines two items into one. Called repeatedly in a tree pattern.
///
/// # Returns
/// * `Some(T)` - The final reduced item if the list is non-empty.
/// * `None` - If the input list is empty.
///
/// # Examples
///
/// **Combine multiple polygons using boolean union**
/// ```rust,no_run
/// use ::geo::BooleanOps;
/// use bevy_ecs_tiled::prelude::*;
///
/// let polygon1 = geo::MultiPolygon::<f32>::empty();
/// let polygon2 = geo::MultiPolygon::<f32>::empty();
/// let polygon3 = geo::MultiPolygon::<f32>::empty();
///
/// let polygons = vec![polygon1, polygon2, polygon3];
/// let combined = simplify_geometry(polygons, |a, b| {
///     // Combine two polygons using boolean union
///     a.union(&b)
/// });
/// ```
pub fn simplify_geometry<T>(list: Vec<T>, mut reduction: impl FnMut(T, T) -> T) -> Option<T> {
    let mut queue = VecDeque::from(list);
    while queue.len() > 1 {
        for _ in 0..(queue.len() / 2) {
            let (one, two) = (queue.pop_front().unwrap(), queue.pop_front().unwrap());
            queue.push_back(reduction(one, two));
        }
    }
    queue.pop_back()
}
