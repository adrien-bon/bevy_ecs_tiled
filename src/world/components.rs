//! This module contains all map [Component]s definition.

use bevy::{prelude::*, utils::HashMap};

/// [Component] holding Tiled world chunking configuration.
///
/// If this value is None, we won't perform chunking: all maps from this world will just be loaded
/// If this value is set, defines the area (in pixel) around each [Camera] where we should spawn a
/// map if it overlaps with its associated [Rect].
///
/// Must be added to the [Entity] holding the world.
#[derive(Component, Default, Reflect)]
pub struct TiledWorldChunking(pub Option<Vec2>);

impl TiledWorldChunking {
    pub fn new(width: f32, height: f32) -> Self {
        Self(Some(Vec2::new(width, height)))
    }
}

/// Marker [Component] for a Tiled world.
#[derive(Component)]
pub struct TiledWorldMarker;

/// Marker [Component] to trigger a world respawn.
///
/// Must be added to the [Entity] holding the map.
#[derive(Component)]
pub struct RespawnTiledWorld;

/// [Component] storing all the Tiled maps that are composing this world.
/// Makes the association between Tiled ID and corresponding Bevy [Entity].
///
/// Should not be manually inserted but can be accessed from the world [Entity].
#[derive(Component, Default, Reflect)]
pub struct TiledWorldStorage {
    /// Map of maps entities, using the map index from [super::asset::TiledWorld]
    /// maps list as key.
    pub spawned_maps: HashMap<usize, Entity>,
}
