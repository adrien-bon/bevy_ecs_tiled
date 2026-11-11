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
    /// * `source` - The kind of Tiled item the collider is created for.
    /// * `origin` - The [`Entity`] of the Tiled item (layer or object) the collider is created for.
    /// * `multi_polygons_list` - The [`geo::MultiPolygon<f32>`] geometries representing the collider shapes.
    ///
    /// # Returns
    /// The [`Entity`] holding the spawned collider(s), which can be `origin` itself.
    /// If the provided collider is not supported, the function should return [`None`].
    ///
    /// # Colliders updates
    /// Colliders are spawned again when their source changes (for instance when a tile of the layer is
    /// added, modified or removed): previous collider entities are then despawned before calling this function.
    ///
    /// If colliders are inserted on `origin` itself, as the built-in backends do for the first collider,
    /// this entity is obviously not despawned: [`TiledPhysicsBackend::remove_colliders`] is called instead
    /// and must remove the physics components this function inserted.
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        source: TiledColliderSource,
        origin: Entity,
        multi_polygons_list: Vec<geo::MultiPolygon<f32>>,
    ) -> Option<Entity>;

    /// Removes the physics components inserted on `origin` itself by [`TiledPhysicsBackend::spawn_colliders`].
    ///
    /// This function is called before spawning again the colliders of `origin`, or when it does not
    /// produce any collider anymore (eg. all the tiles of a layer have been despawned), if
    /// [`TiledPhysicsBackend::spawn_colliders`] returned `origin`. Colliders spawned on other entities are
    /// automatically despawned.
    ///
    /// The default implementation does nothing: it is only suitable for backends which never
    /// insert components on `origin`.
    fn remove_colliders(&self, _commands: &mut Commands, _origin: Entity) {}
}

pub(crate) fn plugin(_app: &mut App) {
    #[cfg(feature = "avian")]
    _app.register_type::<avian::TiledPhysicsAvianBackend>();
    #[cfg(feature = "rapier")]
    _app.register_type::<rapier::TiledPhysicsRapierBackend>();
}
