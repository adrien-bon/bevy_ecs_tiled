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

/// Converts a [`geo::MultiPolygon<f32>`] into a vector of triangles and their centroids.
///
/// Each triangle is represented as an array of three [`Vec2`] points, and its centroid as a [`Vec2`].
/// This is useful for physics backends that require triangulated shapes.
///
/// # Arguments
/// * `multi_polygon` - The input geometry to triangulate.
///
/// # Returns
/// A vector of tuples: ([triangle_vertices; 3], centroid).
pub fn multi_polygon_as_triangles(
    multi_polygon: &geo::MultiPolygon<f32>,
) -> Vec<([Vec2; 3], Vec2)> {
    multi_polygon
        .constrained_triangulation(Default::default())
        .unwrap()
        .into_iter()
        .map(|tri| {
            let (c_x, c_y) = tri.centroid().0.x_y();
            let d = Vec2::new(c_x, c_y);
            let tri = tri.to_array().map(|p| Vec2::new(p.x, p.y)).map(|p| p - d);

            (tri, d)
        })
        .collect()
}

/// Converts a [`geo::MultiPolygon<f32>`] into a vector of [`geo::LineString<f32>`].
///
/// This function extracts all exterior and interior rings from the input geometry and returns them as line strings.
/// Useful for physics backends that operate on polylines or linestrips.
///
/// # Arguments
/// * `multi_polygon` - The input geometry to extract line strings from.
///
/// # Returns
/// A vector of [`geo::LineString<f32>`] representing all rings in the geometry.
pub fn multi_polygon_as_line_strings(
    multi_polygon: &geo::MultiPolygon<f32>,
) -> Vec<geo::LineString<f32>> {
    let mut out = vec![];
    multi_polygon.iter().for_each(|p| {
        [p.interiors(), &[p.exterior().clone()]]
            .concat()
            .into_iter()
            .for_each(|ls| {
                out.push(ls);
            });
    });
    out
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
/// use bevy_ecs_tiled::prelude::{*, geo::BooleanOps};
///
/// let polygon1: geo::MultiPolygon<f32> = geo::MultiPolygon::empty();
/// let polygon2: geo::MultiPolygon<f32> = geo::MultiPolygon::empty();
/// let polygon3: geo::MultiPolygon<f32> = geo::MultiPolygon::empty();
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
