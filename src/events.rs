//! Events related to Tiled map loading
//!
//! These events will be fired after the whole map has loaded.
//! More informations in the [dedicated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/map_events.rs)

use crate::prelude::*;
use bevy::prelude::*;
use tiled::{Layer, LayerTile, Map, Object};

/// Event sent when a Tiled map has finished loading
#[derive(Event, Clone, Debug)]
pub struct TiledMapCreated {
    /// Spawned map [Entity]
    pub map: Entity,
    /// Handle to the loaded Tiled Map
    pub map_handle: Handle<TiledMap>,
}

impl<'a> TiledMapCreated {
    /// Retrieve the [Map] associated to this [TiledMapCreated] event.
    pub fn map(&self, map_asset: &'a Res<Assets<TiledMap>>) -> &'a Map {
        &map_asset.get(self.map_handle.id()).unwrap().map
    }
}

/// Event sent when a Tiled layer has finished loading
#[derive(Event, Clone, Debug)]
pub struct TiledLayerCreated {
    /// Spawned map [Entity]
    pub map: Entity,
    /// Spawned layer [Entity]
    pub layer: Entity,
    /// Handle to the loaded [TiledMap]
    pub map_handle: Handle<TiledMap>,
    /// Layer ID
    pub layer_id: usize,
}

impl<'a> TiledLayerCreated {
    /// Retrieve the [Map] associated to this [TiledLayerCreated] event.
    pub fn map(&self, map_asset: &'a Res<Assets<TiledMap>>) -> &'a Map {
        &map_asset.get(self.map_handle.id()).unwrap().map
    }

    /// Retrieve the [Layer] associated to this [TiledLayerCreated] event.
    pub fn layer(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Layer<'a> {
        self.map(map_asset).get_layer(self.layer_id).unwrap()
    }
}

/// Event sent when a Tiled object has finished loading
#[derive(Event, Clone, Debug)]
pub struct TiledObjectCreated {
    /// Spawned map [Entity]
    pub map: Entity,
    /// Spawned layer [Entity]
    pub layer: Entity,
    /// Spawned object [Entity]
    pub object: Entity,
    /// Handle to the loaded [TiledMap]
    pub map_handle: Handle<TiledMap>,
    /// Layer ID
    pub layer_id: usize,
    /// Object ID
    pub object_id: usize,
}

impl TiledObjectCreated {
    pub fn from_layer(layer: &TiledLayerCreated, object: Entity, object_id: usize) -> Self {
        Self {
            map: layer.map,
            layer: layer.layer,
            layer_id: layer.layer_id,
            map_handle: layer.map_handle.clone(),
            object,
            object_id,
        }
    }
}

impl<'a> TiledObjectCreated {
    /// Retrieve the [Map] associated to this [TiledObjectCreated] event.
    pub fn map(&self, map_asset: &'a Res<Assets<TiledMap>>) -> &'a Map {
        &map_asset.get(self.map_handle.id()).unwrap().map
    }

    /// Retrieve the [Layer] associated to this [TiledObjectCreated] event.
    pub fn layer(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Layer<'a> {
        self.map(map_asset).get_layer(self.layer_id).unwrap()
    }

    /// Retrieve the [Object] associated to this [TiledObjectCreated] event.
    pub fn object(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Object<'a> {
        self.layer(map_asset)
            .as_object_layer()
            .unwrap()
            .get_object(self.object_id)
            .unwrap()
    }
}

/// Event sent when a Tiled special tile has finished loading
///
/// Special tile means it either contains custom properties or physics colliders.
#[derive(Event, Clone, Debug)]
pub struct TiledSpecialTileCreated {
    /// Spawned map [Entity]
    pub map: Entity,
    /// Spawned layer [Entity]
    pub layer: Entity,
    /// Spawned layer for tileset [Entity]
    pub layer_for_tileset: Entity,
    /// Spawned tile [Entity]
    pub tile: Entity,
    /// Handle to the loaded [TiledMap]
    pub map_handle: Handle<TiledMap>,
    /// Layer ID
    pub layer_id: usize,
    /// Tile X position in the layer
    pub x: i32,
    /// Tile Y position in the layer
    pub y: i32,
    /// Tile absolute position in Bevy space
    pub position: Vec2,
}

impl TiledSpecialTileCreated {
    pub fn from_layer(
        layer: &TiledLayerCreated,
        layer_for_tileset: Entity,
        tile: Entity,
        x: i32,
        y: i32,
        position: Vec2,
    ) -> Self {
        Self {
            map: layer.map,
            layer: layer.layer,
            layer_id: layer.layer_id,
            map_handle: layer.map_handle.clone(),
            layer_for_tileset,
            tile,
            x,
            y,
            position,
        }
    }
}

impl<'a> TiledSpecialTileCreated {
    /// Retrieve the [Map] associated to this [TiledSpecialTileCreated] event.
    pub fn map(&self, map_asset: &'a Res<Assets<TiledMap>>) -> &'a Map {
        &map_asset.get(self.map_handle.id()).unwrap().map
    }

    /// Retrieve the [Layer] associated to this [TiledSpecialTileCreated] event.
    pub fn layer(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Layer<'a> {
        self.map(map_asset).get_layer(self.layer_id).unwrap()
    }

    /// Retrieve the [LayerTile] associated to this [TiledSpecialTileCreated] event.
    pub fn tile(&self, map_asset: &'a Res<Assets<TiledMap>>) -> LayerTile<'a> {
        self.layer(map_asset)
            .as_tile_layer()
            .unwrap()
            .get_tile(self.x, self.y)
            .unwrap()
    }
}
