//! Avian physics backend for `bevy_ecs_tiled`.
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

use crate::{
    physics::geometry::{TiledColliderShape, collider_shapes},
    prelude::*,
};
use avian2d::{
    parry::{math::Pose, shape::SharedShape},
    prelude::*,
};
use bevy::prelude::*;

/// The [`TiledPhysicsBackend`] to use for Avian 2D integration.
///
/// How colliders are created from Tiled shapes is selected using a [`TiledColliderStrategy`]
/// (see [`TiledPhysicsSettings::collider_strategy`]):
/// - [`TiledColliderStrategy::Polyline`]: a single collider, made of a [`SharedShape::polyline`].
/// - [`TiledColliderStrategy::Triangulation`]: a single collider, made of a [`SharedShape::compound`] of [`SharedShape::triangle`]s.
/// - [`TiledColliderStrategy::LineStrip`]: several colliders, each one made of a [`SharedShape::polyline`].
#[derive(Default, Reflect, Copy, Clone, Debug)]
#[reflect(Default, Debug)]
pub struct TiledPhysicsAvianBackend;

impl TiledPhysicsBackend for TiledPhysicsAvianBackend {
    type Collider = Collider;

    fn colliders(
        &self,
        strategy: TiledColliderStrategy,
        _source: TiledColliderSource,
        geometry: &TiledColliderGeometry,
    ) -> Vec<Self::Collider> {
        collider_shapes(strategy, geometry)
            .into_iter()
            .map(|shape| match shape {
                TiledColliderShape::Polyline { vertices, indices } => {
                    SharedShape::polyline(vertices, indices).into()
                }
                TiledColliderShape::Triangles(triangles) => SharedShape::compound(
                    triangles
                        .into_iter()
                        .map(|([a, b, c], centroid)| {
                            (Pose::new(centroid, 0.), SharedShape::triangle(a, b, c))
                        })
                        .collect(),
                )
                .into(),
            })
            .collect()
    }
}
