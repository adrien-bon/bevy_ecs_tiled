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
/// use bevy_ecs_tiled::prelude::*;
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
///         geometry: &TiledColliderGeometry,
///     ) -> Vec<Self::Collider> {
///         vec![MyCollider(geometry.polygons.0.len())]
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

    /// Creates physics colliders for the given geometry, of the given `source` kind.
    ///
    /// The `geometry` is relative to the Tiled object or tiles layer entity.
    /// When the colliders of a tiles layer are split in chunks, it also holds the geometry of the adjacent
    /// chunks: to prevent bodies from getting stuck at chunk junctions, colliders made of segments should only
    /// use its [`outline`](TiledColliderGeometry::outline), which does not include the edges shared with them.
    /// The `strategy` tells how colliders should be created from it: a backend which does not support
    /// all strategies is free to fall back to another one.
    /// It is never [`TiledColliderStrategy::Disabled`]: no collider is created in this case.
    ///
    /// Usually returns a single collider, but there can be several (see [`TiledColliderStrategy::LineStrip`])
    /// or none if the geometry is not supported.
    ///
    /// See [`collider_shapes`](crate::physics::geometry::collider_shapes) for an helper converting the geometry to shapes
    /// which are easy to map to physics engines primitives.
    fn colliders(
        &self,
        strategy: TiledColliderStrategy,
        source: TiledColliderSource,
        geometry: &TiledColliderGeometry,
    ) -> Vec<Self::Collider>;
}

pub(crate) fn plugin(_app: &mut App) {
    #[cfg(feature = "avian")]
    _app.register_type::<avian::TiledPhysicsAvianBackend>();
    #[cfg(feature = "rapier")]
    _app.register_type::<rapier::TiledPhysicsRapierBackend>();
}
