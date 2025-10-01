//! Storage structures for Tiled map data.
//!
//! This module defines data structures and utilities for storing and managing the contents of a Tiled map,
//! including layers, tiles, and associated metadata. It provides efficient access and organization of map data
//! for use by systems and plugins within the bevy_ecs_tiled framework.

use crate::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

/// [`Component`] storing all the Tiled items composing this map.
/// Makes the association between Tiled ID and corresponding Bevy [`Entity`].
///
/// Should not be manually inserted but can be accessed from the map [`Entity`].
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledMapStorage {
    /// Mapping between a Tiled layer ID with corresponding [`TiledLayer`] [`Entity`]
    pub(crate) layers: HashMap<u32, Entity>,

    /// Mapping between a Tiled object ID with corresponding [`TiledObject`] [`Entity`]
    pub(crate) objects: HashMap<u32, Entity>,

    /// Mapping between a Tiled tileset ID + [`TileId`] with all corresponding [`TiledTile`] [`Entity`]s
    ///
    /// Note that we can have multiple entities (ie.several instances) of the same tile since
    /// it references the tile on the tileset and not the tile on the tilemap.
    pub(crate) tiles: HashMap<(u32, tiled::TileId), Vec<Entity>>,
}

impl TiledMapStorage {
    /// Clear the [`TiledMapStorage`], removing all children layers in the process
    pub fn clear(&mut self, commands: &mut Commands) {
        for layer_entity in self.layers.values() {
            commands.entity(*layer_entity).despawn();
        }
        self.layers.clear();
        self.objects.clear();
        self.tiles.clear();
    }
}

impl<'a> TiledMapStorage {
    /// Returns an iterator over the [`TiledLayer`] [`Entity`] and layer ID associations
    pub fn layers(&self) -> bevy::platform::collections::hash_map::Iter<'_, u32, Entity> {
        self.layers.iter()
    }

    /// Retrieve the [`TiledLayer`] [`Entity`] associated with this layer ID
    pub fn get_layer_entity(&self, layer_id: u32) -> Option<Entity> {
        self.layers.get(&layer_id).cloned()
    }

    /// Retrieve the layer ID associated with this [`TiledLayer`] [`Entity`]
    pub fn get_layer_id(&self, entity: Entity) -> Option<u32> {
        self.layers
            .iter()
            .find(|(_, &e)| e == entity)
            .map(|(&id, _)| id)
    }

    /// Retrieve the [`Layer`] associated with this [`TiledLayer`] [`Entity`]
    pub fn get_layer(&self, map: &'a tiled::Map, entity: Entity) -> Option<tiled::Layer<'a>> {
        self.get_layer_id(entity)
            .and_then(|id| get_layer_from_map(map, id))
    }

    /// Returns an iterator over the [`TiledTile`] [`Entity`] and tileset ID + [`TileId`] associations
    pub fn tiles(
        &self,
    ) -> bevy::platform::collections::hash_map::Iter<'_, (u32, tiled::TileId), Vec<Entity>> {
        self.tiles.iter()
    }

    /// Retrieve the [`TiledTile`] [`Entity`] list associated with this tileset ID and [`TileId`]
    pub fn get_tile_entities(&self, tileset_id: u32, tile_id: tiled::TileId) -> Vec<Entity> {
        self.tiles
            .get(&(tileset_id, tile_id))
            .cloned()
            .unwrap_or_default()
    }

    /// Retrieve the tileset ID and [`TileId`] associated with this [`TiledTile`] [`Entity`]
    pub fn get_tile_id(&self, entity: Entity) -> Option<(u32, tiled::TileId)> {
        self.tiles
            .iter()
            .find(|(_, v)| v.contains(&entity))
            .map(|(&id, _)| id)
    }

    /// Retrieve the [`Tile`] associated with this [`TiledTile`] [`Entity`]
    pub fn get_tile(&self, map: &'a tiled::Map, entity: Entity) -> Option<tiled::Tile<'a>> {
        self.get_tile_id(entity)
            .and_then(|(tileset_id, tile_id)| get_tile_from_map(map, tileset_id, tile_id))
    }

    /// Returns an iterator over the [`TiledObject`] [`Entity`] and object ID associations
    pub fn objects(&self) -> bevy::platform::collections::hash_map::Iter<'_, u32, Entity> {
        self.objects.iter()
    }

    /// Retrieve the [`TiledObject`] [`Entity`] associated with this object ID
    pub fn get_object_entity(&self, object_id: u32) -> Option<Entity> {
        self.objects.get(&object_id).cloned()
    }

    /// Retrieve the object ID associated with this [`TiledObject`] [`Entity`]
    pub fn get_object_id(&self, entity: Entity) -> Option<u32> {
        self.objects
            .iter()
            .find(|(_, &e)| e == entity)
            .map(|(&id, _)| id)
    }

    /// Retrieve the [`Object`] associated with this [`TiledObject`] [`Entity`]
    pub fn get_object(&self, map: &'a tiled::Map, entity: Entity) -> Option<tiled::Object<'a>> {
        self.get_object_id(entity)
            .and_then(|id| get_object_from_map(map, id))
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledMapStorage>();
}
