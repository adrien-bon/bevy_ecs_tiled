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

use std::fmt;

use bevy::{prelude::*, reflect::Reflectable};
use bevy_ecs_tilemap::anchor::TilemapAnchor;

use crate::{
    names::TiledNameFilter,
    tiled::{event::TiledEvent, map::asset::TiledMapAsset},
};

use super::collider::ColliderCreated;

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
    /// # Arguments
    /// * `commands` - The Bevy [`Commands`] instance for spawning entities.
    /// * `assets` - Reference to the loaded [`TiledMapAsset`] assets.
    /// * `anchor` - The anchor point of the tilemap.
    /// * `filter` - Name filter for selecting which objects/tiles to spawn colliders for.
    /// * `source` - The event describing the collider to be created.
    ///
    /// # Returns
    /// A vector of [`TiledPhysicsBackendOutput`] describing the spawned colliders.
    ///
    /// If the provided collider is not supported, the function should return an empty vector.
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        assets: &Res<Assets<TiledMapAsset>>,
        anchor: &TilemapAnchor,
        filter: &TiledNameFilter,
        source: &TiledEvent<ColliderCreated>,
    ) -> Vec<TiledPhysicsBackendOutput>;
}

/// Output information for a spawned physics collider.
///
/// This struct contains details about a collider entity created by a physics backend,
/// including its name, entity ID, and transform relative to its parent.
#[derive(Clone, Debug)]
pub struct TiledPhysicsBackendOutput {
    /// Name of the collider.
    pub name: String,
    /// [`Entity`] of the spawned collider.
    pub entity: Entity,
    /// Relative position and rotation of the collider from its parent [`Entity`].
    pub transform: Transform,
}

pub(crate) fn plugin(app: &mut App) {
    #[cfg(feature = "avian")]
    app.register_type::<avian::TiledPhysicsAvianBackend>();
    #[cfg(feature = "rapier")]
    app.register_type::<rapier::TiledPhysicsRapierBackend>();
}
