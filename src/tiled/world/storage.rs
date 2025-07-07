//! Storage structures for Tiled world data.
//!
//! This module provides data structures and utilities for storing and managing Tiled world information,
//! including references to maps, world chunks, and world-level metadata. It enables efficient access and
//! organization of world data for chunking, streaming, and world management systems.

use crate::prelude::*;
use bevy::prelude::*;

/// [`Component`] storing all the Tiled maps that are composing this world.
/// Makes the association between Tiled ID and corresponding Bevy [`Entity`].
///
/// Should not be manually inserted but can be accessed from the world [`Entity`].
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledWorldStorage {
    /// Mapping between a Tiled map ID with corresponding [`TiledMap`] [`Entity`]
    pub(crate) maps: HashMap<u32, Entity>,
}

impl TiledWorldStorage {
    /// Clear the [`TiledWorldStorage`], removing all children maps in the process
    pub fn clear(&mut self, commands: &mut Commands) {
        for (_, map_entity) in self.maps.iter() {
            commands.entity(*map_entity).despawn();
        }
        self.maps.clear();
    }

    /// Returns an iterator over the [`TiledMap`] [`Entity`] and map ID associations
    pub fn maps(&self) -> bevy::platform::collections::hash_map::Iter<'_, u32, Entity> {
        self.maps.iter()
    }

    /// Retrieve the [`TiledMap`] [`Entity`] associated with this map ID
    pub fn get_map_entity(&self, map_id: u32) -> Option<Entity> {
        self.maps.get(&map_id).cloned()
    }

    /// Retrieve the map ID associated with this [`TiledMap`] [`Entity`]
    pub fn get_map_id(&self, entity: Entity) -> Option<u32> {
        self.maps
            .iter()
            .find(|(_, &e)| e == entity)
            .map(|(&id, _)| id)
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledWorldStorage>();
}
