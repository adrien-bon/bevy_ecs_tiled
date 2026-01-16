//! Physics backend abstraction for Tiled maps and worlds.
//!
//! This module defines the [`TiledPhysicsBackend`] trait, which must be implemented by any custom physics backend
//! to support physics collider generation for Tiled maps and worlds.
//!
//! Built-in support is provided for Rapier and Avian backends via feature flags.

// #[cfg(feature = "rapier")]
// pub mod rapier;

// #[cfg(feature = "avian")]
// pub mod avian;

use std::fmt;

use crate::prelude::{
    geo::{Centroid, TriangulateDelaunay},
    *,
};
use bevy::{prelude::*, reflect::Reflectable};

/// Trait for implementing a custom physics backend for Tiled maps and worlds.
///
/// Any physics backend must implement this trait to support spawning colliders for Tiled objects and tiles.
/// The backend is responsible for creating the appropriate physics entities and returning information about them.
pub trait TiledPhysicsBackend:
    Default
    + Clone
    + fmt::Debug
    + 'static
    + std::marker::Sync
    + std::marker::Send
    + FromReflect
    + Reflectable
{
    /// Spawns one or more physics colliders for a given Tiled object or tile layer.
    ///
    /// This function is called by the physics integration to generate colliders for Tiled objects or tiles.
    /// The backend implementation is responsible for creating the appropriate physics entities and returning
    /// information about them.
    ///
    /// # Arguments
    /// * `commands` - The Bevy [`Commands`] instance for spawning entities.
    /// * `source` - The event describing the collider to be created.
    /// * `multi_polygon` - The [`geo::MultiPolygon<f32>`] geometry representing the collider shape.
    ///
    /// # Returns
    /// A vector of [`Entity`] of the spawned colliders.
    /// If the provided collider is not supported, the function should return an empty vector.
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        source: &TiledEvent<ColliderCreated>,
        multi_polygon: &geo::MultiPolygon<f32>,
    ) -> Vec<Entity>;
}

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

pub(crate) fn plugin(_app: &mut App) {
    // #[cfg(feature = "avian")]
    // _app.register_type::<avian::TiledPhysicsAvianBackend>();
    // #[cfg(feature = "rapier")]
    // _app.register_type::<rapier::TiledPhysicsRapierBackend>();
}
