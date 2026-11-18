//! Physics backend abstraction for Tiled maps and worlds.
//!
//! This module defines the [`TiledPhysicsBackend`] trait, which must be implemented by any custom physics backend
//! to support physics collider generation for Tiled maps and worlds.
//!
//! Built-in support is provided for Rapier and Avian backends via feature flags.

#[cfg(feature = "rapier")]
pub mod rapier;

#[cfg(feature = "avian")]
pub mod avian;

use bevy::{prelude::*, reflect::Reflectable};
use std::fmt;

use crate::prelude::*;

/// Trait for implementing a custom physics backend for Tiled maps and worlds.
///
/// A physics backend only creates physics colliders out of a geometry: `bevy_ecs_tiled` takes care of
/// computing this geometry from Tiled objects and tiles, of inserting the colliders on the
/// right entities and of removing them when their source changes.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::{geo, prelude::*};
///
/// /// A "collider" which only holds the number of polygons of its geometry
/// #[derive(Component)]
/// struct MyCollider(usize);
///
/// #[derive(Default, Debug, Clone, Reflect)]
/// struct MyBackend;
///
/// impl TiledPhysicsBackend for MyBackend {
///     type Collider = MyCollider;
///
///     fn colliders(
///         &self,
///         _strategy: TiledColliderStrategy,
///         _source: TiledColliderSource,
///         geometry: &geo::MultiPolygon<f32>,
///     ) -> Vec<Self::Collider> {
///         vec![MyCollider(geometry.0.len())]
///     }
/// }
/// ```
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
    /// The components making a physics collider for this backend.
    ///
    /// The first collider of a Tiled object or tiles layer is inserted on the object or layer entity
    /// itself (so it is easy to add other physics components to it, such as a rigid body), and the next ones,
    /// if any, on dedicated child entities.
    ///
    /// When the source of a collider changes (for instance when a tile of the layer is added, modified
    /// or removed), child entities are despawned and this bundle is removed from the object or layer
    /// entity, before creating colliders again.
    /// Components which are not part of this bundle are not removed: if you need other components to
    /// follow the collider (eg. a mesh to display it), use component hooks or observers.
    type Collider: Bundle;

    /// Creates physics colliders for the given geometry.
    ///
    /// # Arguments
    /// * `strategy` - How colliders should be created from the geometry. A backend which does not support
    ///   all strategies is free to fall back to another one.
    /// * `source` - The kind of Tiled item the colliders are created for.
    /// * `geometry` - The [`geo::MultiPolygon<f32>`] representing the colliders shape, relative to
    ///   the Tiled object or tiles layer entity.
    ///
    /// # Returns
    /// The colliders to spawn for this geometry: usually a single one, but there can be several
    /// (see [`TiledColliderStrategy::LineStrip`]) or none if the geometry is not supported.
    ///
    /// See [`collider_shapes`](crate::physics::geometry::collider_shapes) for an helper converting the geometry to shapes
    /// which are easy to map to physics engines primitives.
    fn colliders(
        &self,
        strategy: TiledColliderStrategy,
        source: TiledColliderSource,
        geometry: &geo::MultiPolygon<f32>,
    ) -> Vec<Self::Collider>;
}

pub(crate) fn plugin(_app: &mut App) {
    #[cfg(feature = "avian")]
    _app.register_type::<avian::TiledPhysicsAvianBackend>();
    #[cfg(feature = "rapier")]
    _app.register_type::<rapier::TiledPhysicsRapierBackend>();
}
