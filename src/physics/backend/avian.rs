//! Avian physics backend for bevy_ecs_tiled.
//!
//! This module provides an implementation of the [`TiledPhysicsBackend`] trait using the Avian 2D physics engine.
//! This backend is only available when the `avian` feature is enabled.
//!
//! # Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_ecs_tiled::prelude::*;
//!
//! App::new()
//!     .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default());
//! ```
//!

use crate::prelude::*;
use avian2d::{
    parry::{
        math::{Isometry, Point, Real},
        shape::SharedShape,
    },
    prelude::*,
};
use bevy::prelude::*;

/// The [`TiledPhysicsBackend`] to use for Avian 2D integration.
///
/// This enum allows you to select how colliders are generated from the input [`geo::MultiPolygon`]s list:
/// - [`TiledPhysicsAvianBackend::Polyline`]: Convert each [`geo::MultiPolygon`] into several [`geo::LineString`]s and aggregate them all into a single polyline collider.
/// - [`TiledPhysicsAvianBackend::Triangulation`]: Convert each [`geo::MultiPolygon`] into several triangles then into a compound collider.
/// - [`TiledPhysicsAvianBackend::LineStrip`]: Convert each [`geo::MultiPolygon`] into several [`geo::LineString`]s and create a linestrip collider for each one of them.
#[derive(Default, Reflect, Copy, Clone, Debug)]
#[reflect(Default, Debug)]
pub enum TiledPhysicsAvianBackend {
    #[default]
    /// Create a single collider from a [`SharedShape::polyline`].
    Polyline,
    /// Performs triangulation and create a single collider by aggregating multiple [`SharedShape::triangle`]s.
    Triangulation,
    /// Produces several colliders from multiple [`SharedShape::polyline`].
    LineStrip,
}

impl TiledPhysicsBackend for TiledPhysicsAvianBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        _source: TiledColliderSource,
        origin: Entity,
        multi_polygons_list: Vec<geo::MultiPolygon<f32>>,
    ) -> Option<Entity> {
        let mut out = None;
        for multi_polygon in multi_polygons_list {
            match self {
                TiledPhysicsAvianBackend::Triangulation => {
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
                        let collider_entity = match out {
                            Some(parent) => commands
                                .spawn((ChildOf(parent), Name::from("Avian[Triangulation]")))
                                .id(),
                            None => {
                                out = Some(origin);
                                origin
                            }
                        };
                        commands.entity(collider_entity).insert(collider);
                    }
                }
                TiledPhysicsAvianBackend::LineStrip => {
                    multi_polygon_as_line_strings(&multi_polygon)
                        .iter()
                        .enumerate()
                        .for_each(|(i, ls)| {
                            let collider: Collider = SharedShape::polyline(
                                ls.points().map(|v| Point::new(v.x(), v.y())).collect(),
                                None,
                            )
                            .into();
                            let collider_entity = match out {
                                Some(parent) => commands
                                    .spawn((
                                        ChildOf(parent),
                                        Name::from(format!("Avian[LineStrip {i}]")),
                                    ))
                                    .id(),
                                None => {
                                    out = Some(origin);
                                    origin
                                }
                            };
                            commands.entity(collider_entity).insert(collider);
                        });
                }
                TiledPhysicsAvianBackend::Polyline => {
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
                        let collider: Collider =
                            SharedShape::polyline(vertices, Some(indices)).into();
                        let collider_entity = match out {
                            Some(parent) => commands
                                .spawn((ChildOf(parent), Name::from("Avian[Polyline]")))
                                .id(),
                            None => {
                                out = Some(origin);
                                origin
                            }
                        };
                        commands.entity(collider_entity).insert(collider);
                    }
                }
            }
        }
        out
    }
}
