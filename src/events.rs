//! Events related to Tiled map loading

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::{ObjectData, TileData};

use crate::prelude::*;

/// Event sent when a Tiled map has finished loading
#[derive(Event, Clone, Debug)]
pub struct TiledMapCreated {
    /// Spawned map `Entity`
    pub map: Entity,
    /// Map type
    pub map_type: TilemapType,
    /// Map size, expressed in number of tiles
    pub map_size: TilemapSize,
    /// Tile size, expressed in pixels
    pub grid_size: TilemapGridSize,
}

/// Event sent when a Tiled layer has finished loading
#[derive(Event, Clone, Debug)]
pub struct TiledLayerCreated {
    /// Spawned layer `Entity`
    pub layer: Entity,
    /// Spawned map `Entity`
    pub map: Entity,
    /// Map type
    pub map_type: TilemapType,
    /// Map size, expressed in number of tiles
    pub map_size: TilemapSize,
    /// Tile size, expressed in pixels
    pub grid_size: TilemapGridSize,
}

/// Event sent when a Tiled object has finished loading
#[derive(Event, Clone, Debug)]
pub struct TiledObjectCreated {
    /// Spawned object `Entity`
    pub object: Entity,
    /// Tiled object data
    pub object_data: ObjectData,
    /// Spawned layer `Entity`
    pub layer: Entity,
    /// Spawned map `Entity`
    pub map: Entity,
    /// Map type
    pub map_type: TilemapType,
    /// Map size, expressed in number of tiles
    pub map_size: TilemapSize,
    /// Tile size, expressed in pixels
    pub grid_size: TilemapGridSize,
}

#[cfg(feature = "physics")]
impl TiledObjectCreated {
    /// Automatically spawn physics colliders associated to this object
    pub fn spawn_collider(
        &self,
        mut commands: Commands,
        physics_backend: PhysicsBackend,
        collider_callback: ColliderCallback,
    ) {
        physics_backend.insert_object_colliders(
            &mut commands,
            self.object,
            &self.map_type,
            &self.object_data,
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
    pub tile: Entity,
    /// Tiled object data
    pub tile_data: TileData,
    /// Spawned layer `Entity`
    pub layer: Entity,
    /// Spawned map `Entity`
    pub map: Entity,
    /// Map type
    pub map_type: TilemapType,
    /// Map size, expressed in number of tiles
    pub map_size: TilemapSize,
    /// Tile size, expressed in pixels
    pub grid_size: TilemapGridSize,
}

#[cfg(feature = "physics")]
impl TiledSpecialTileCreated {
    /// Automatically spawn physics colliders associated to this tile
    ///
    /// Note you must provide a custom [ObjectNames] filter to select which objects your want to add colliders for.
    pub fn spawn_collider(
        &self,
        mut commands: Commands,
        physics_backend: PhysicsBackend,
        collision_object_names: ObjectNames,
        collider_callback: ColliderCallback,
    ) {
        if let Some(collision) = &self.tile_data.collision {
            physics_backend.insert_tile_colliders(
                &mut commands,
                &ObjectNameFilter::from(&collision_object_names),
                self.tile,
                &self.map_type,
                &self.grid_size,
                collision,
                collider_callback,
            )
        }
    }
}
