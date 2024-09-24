//! Events related to Tiled map loading

use bevy::prelude::*;
use tiled::{Layer, LayerTile, Map, Object};
use crate::prelude::*;

/// Event sent when a Tiled map has finished loading
#[derive(Event, Clone, Debug)]
pub struct TiledMapCreated {
    /// Spawned map `Entity`
    pub entity: Entity,
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
    /// Spawned layer `Entity`
    pub entity: Entity,
    /// Handle to the loaded Tiled Map
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
    /// Spawned object `Entity`
    pub entity: Entity,
    /// Handle to the loaded Tiled Map
    pub map_handle: Handle<TiledMap>,
    /// Layer ID
    pub layer_id: usize,
    /// Object ID
    pub object_id: usize,
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
        self.layer(map_asset).as_object_layer().unwrap().get_object(self.object_id).unwrap()
    }
}

#[cfg(feature = "physics")]
impl TiledObjectCreated {
    /// Automatically spawn physics colliders associated to this object
    pub fn spawn_collider(
        &self,
        map_asset: &Res<Assets<TiledMap>>,
        mut commands: Commands,
        physics_backend: PhysicsBackend,
        collider_callback: ColliderCallback,
    ) {
        physics_backend.insert_object_colliders(
            &mut commands,
            self.entity,
            &get_map_type(self.map(map_asset)),
            &self.object(map_asset),
            collider_callback,
        )
    }
}

/// Event sent when a Tiled special tile has finished loading
///
/// Special tile means it either contains custom properties or physics colliders.
#[derive(Event, Clone, Debug)]
pub struct TiledSpecialTileCreated {
    /// Spawned tile entity
    pub entity: Entity,
    /// Handle to the loaded Tiled Map
    pub map_handle: Handle<TiledMap>,
    /// Layer ID
    pub layer_id: usize,
    /// Tile X position in the layer
    pub x: i32,
    /// Tile Y position in the layer
    pub y: i32
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
        self.layer(map_asset).as_tile_layer().unwrap().get_tile(self.x, self.y).unwrap()
    }
}

#[cfg(feature = "physics")]
impl TiledSpecialTileCreated {
    /// Automatically spawn physics colliders associated to this tile
    ///
    /// Note you must provide a custom [ObjectNames] filter to select which objects your want to add colliders for.
    pub fn spawn_collider(
        &self,
        mut commands: Commands,
        map_asset: &Res<Assets<TiledMap>>,
        physics_backend: PhysicsBackend,
        collision_object_names: ObjectNames,
        collider_callback: ColliderCallback,
    ) {

        if let Some(collision) = &self.tile(map_asset).get_tile().unwrap().collision {
            let map = self.map(map_asset);
            physics_backend.insert_tile_colliders(
                &mut commands,
                &ObjectNameFilter::from(&collision_object_names),
                self.entity,
                &get_map_type(map),
                &get_grid_size(map),
                collision,
                collider_callback,
            )
        }
    }
}
