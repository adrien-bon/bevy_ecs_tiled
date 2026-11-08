//! Storage structures for Tiled world data.
//!
//! This module provides data structures and utilities for storing and managing Tiled world information,
//! including references to maps, world chunks, and world-level metadata. It enables efficient access and
//! organization of world data for chunking, streaming, and world management systems.

#[allow(unused_imports)]
use crate::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

/// ID of a map in a [`TiledWorldAsset`], as its index in the `.world` file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect)]
#[reflect(Clone, Debug, Default, PartialEq, Hash)]
pub struct WorldMapId(pub u32);

crate::tiled::helpers::impl_id!(WorldMapId);

/// [`Component`] storing all the Tiled maps that are composing this world.
/// Makes the association between the [`WorldMapId`] of a map and corresponding Bevy [`Entity`].
///
/// Should not be manually inserted but can be accessed from the world [`Entity`].
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledWorldStorage {
    /// Mapping between a [`WorldMapId`] with corresponding [`TiledMap`] [`Entity`]
    pub(crate) maps: HashMap<WorldMapId, Entity>,
}

impl TiledWorldStorage {
    /// Clear the [`TiledWorldStorage`], removing all children maps in the process
    ///
    /// To respawn a world, insert the [`RespawnTiledWorld`] component on the world entity instead.
    pub(crate) fn clear(&mut self, commands: &mut Commands) {
        for (_, map_entity) in self.maps.iter() {
            commands.entity(*map_entity).despawn();
        }
        self.maps.clear();
    }

    /// Returns an iterator over the [`WorldMapId`] and [`TiledMap`] [`Entity`] associations
    pub fn maps(&self) -> impl Iterator<Item = (WorldMapId, Entity)> + '_ {
        self.maps.iter().map(|(&idx, &entity)| (idx, entity))
    }

    /// Retrieve the [`TiledMap`] [`Entity`] associated with this [`WorldMapId`]
    pub fn map_entity(&self, map_id: WorldMapId) -> Option<Entity> {
        self.maps.get(&map_id).cloned()
    }

    /// Retrieve the [`WorldMapId`] associated with this [`TiledMap`] [`Entity`]
    pub fn map_id(&self, entity: Entity) -> Option<WorldMapId> {
        self.maps
            .iter()
            .find(|(_, e)| **e == entity)
            .map(|(&id, _)| id)
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<WorldMapId>();
    app.register_type::<TiledWorldStorage>();
}
