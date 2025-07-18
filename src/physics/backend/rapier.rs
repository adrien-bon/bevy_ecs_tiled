//! Rapier physics backend for bevy_ecs_tiled.
//!
//! This module provides an implementation of the [`TiledPhysicsBackend`] trait using the Rapier 2D physics engine.
//! This backend is only available when the `rapier` feature is enabled.
//!
//! # Example:
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_ecs_tiled::prelude::*;
//!
//! App::new()
//!     .add_plugins(TiledPhysicsPlugin::<TiledPhysicsRapierBackend>::default());
//! ```

use crate::prelude::*;
use bevy::prelude::*;
use bevy_rapier2d::{
    prelude::*,
    rapier::prelude::{Isometry, Point, Real, SharedShape},
};

/// The [`TiledPhysicsBackend`] to use for Rapier 2D integration.
///
/// This enum allows you to select how colliders are generated from Tiled shapes:
/// - [`Polyline`]: Aggregates all line strings into a single polyline collider.
/// - [`Triangulation`]: Triangulates polygons and aggregates triangles into a compound collider.
/// - [`LineStrip`]: Creates a separate linestrip collider for each line string.
#[derive(Default, Reflect, Copy, Clone, Debug)]
#[reflect(Default, Debug)]
pub enum TiledPhysicsRapierBackend {
    #[default]
    /// Aggregates all [`LineString`]s into a single collider using [`SharedShape::polyline`].
    Polyline,
    /// Performs triangulation and produces a single collider by aggregating multiple [`SharedShape::triangle`]s.
    Triangulation,
    /// Produces several linestrip colliders, one for each line string.
    LineStrip,
}

impl TiledPhysicsBackend for TiledPhysicsRapierBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        _source: &TiledEvent<ColliderCreated>,
        multi_polygon: MultiPolygon<f32>,
    ) -> Vec<TiledPhysicsBackendOutput> {
        let mut out = vec![];
        match self {
            TiledPhysicsRapierBackend::Triangulation => {
                let shared_shapes = multi_polygon_as_triangles(&multi_polygon)
                    .iter()
                    .map(|([a, b, c], centroid)| {
                        (
                            Isometry::<Real>::new((*centroid).into(), 0.),
                            SharedShape::triangle((*a).into(), (*b).into(), (*c).into()),
                        )
                    })
                    .collect::<Vec<_>>();

                if !shared_shapes.is_empty() {
                    let collider: Collider = SharedShape::compound(shared_shapes).into();
                    out.push(TiledPhysicsBackendOutput {
                        name: "Rapier[Trianguation]".to_string(),
                        entity: commands.spawn(collider).id(),
                        transform: Transform::default(),
                    });
                }
            }
            TiledPhysicsRapierBackend::LineStrip => {
                multi_polygon_as_line_strings(&multi_polygon)
                    .iter()
                    .for_each(|ls| {
                        let collider: Collider = SharedShape::polyline(
                            ls.points().map(|v| Point::new(v.x(), v.y())).collect(),
                            None,
                        )
                        .into();
                        out.push(TiledPhysicsBackendOutput {
                            name: "Rapier[LineStrip]".to_string(),
                            entity: commands.spawn(collider).id(),
                            transform: Transform::default(),
                        })
                    });
            }
            TiledPhysicsRapierBackend::Polyline => {
                let mut vertices = vec![];
                let mut indices = vec![];
                multi_polygon_as_line_strings(&multi_polygon)
                    .iter()
                    .for_each(|ls| {
                        ls.lines().for_each(|l| {
                            let points = l.points();
                            let len = vertices.len();
                            vertices.push(Point::new(points.0.x(), points.0.y()));
                            vertices.push(Point::new(points.1.x(), points.1.y()));
                            indices.push([len as u32, (len + 1) as u32]);
                        });
                    });
                if !vertices.is_empty() {
                    let collider: Collider = SharedShape::polyline(vertices, Some(indices)).into();
                    out.push(TiledPhysicsBackendOutput {
                        name: "Rapier[Polyline]".to_string(),
                        entity: commands.spawn(collider).id(),
                        transform: Transform::default(),
                    })
                }
            }
        }
        out
    }
}
