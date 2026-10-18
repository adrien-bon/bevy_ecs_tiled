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
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        source: TiledColliderSource,
        origin: Entity,
        multi_polygons_list: Vec<geo::MultiPolygon<f32>>,
    ) -> Option<Entity>;
}

pub(crate) fn plugin(_app: &mut App) {
    #[cfg(feature = "avian")]
    _app.register_type::<avian::TiledPhysicsAvianBackend>();
    #[cfg(feature = "rapier")]
    _app.register_type::<rapier::TiledPhysicsRapierBackend>();
}
