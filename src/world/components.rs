//! This module contains all map [Component]s definition.

use bevy::{prelude::*, utils::HashMap};

/// [Component] holding Tiled world related settings.
///
/// Controls various settings related to the way we handle the Tiled world.
/// Must be added to the [Entity] holding the world.
#[derive(Component, Default, Reflect)]
pub struct TiledWorldSettings {
    /// World chunking configuration
    ///
    /// If this value is None, we won't perform chunking: all maps from this world will just be loaded
    /// If this value is set, defines the area (in pixel) around each [Camera] where we should spawn a
    /// map if it overlaps with its associated [Rect].
    pub chunking: Option<Vec2>,
}

/// Marker [Component] for a Tiled world.
#[derive(Component)]
pub struct TiledWorldMarker;

/// Marker [Component] to trigger a world respawn.
///
/// Must be added to the [Entity] holding the map.
#[derive(Component)]
pub struct RespawnTiledWorld;

/// [Component] storing informations about which maps are actually spawned
#[derive(Component, Default, Reflect)]
pub struct TiledWorldStorage {
    /// Map using the map index from [super::asset::TiledWorld] maps list as key.
    /// It contains the map entity.
    pub spawned_maps: HashMap<usize, Entity>,
}
