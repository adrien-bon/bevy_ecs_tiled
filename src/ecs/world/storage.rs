//! [`TiledWorldStorage`]: the map entities of a world, by [`WorldMapId`].

#[allow(unused_imports)]
use crate::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

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
        for (_, map_entity) in &self.maps {
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
        self.maps.get(&map_id).copied()
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
    app.register_type::<TiledWorldStorage>();
}
