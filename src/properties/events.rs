use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::{ObjectData, TileData};

#[cfg(feature = "physics")]
use crate::prelude::*;

/// This event is sent when an object registered with [register_tiled_object] is spawned.
/// It should be handled using an observer. See [associated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/user_properties.rs#L99)
#[derive(Event, Clone, Debug)]
pub struct TiledObjectCreated {
    /// Spawned object entity
    pub entity: Entity,
    /// Tiled map type
    pub map_type: TilemapType,
    /// Tiled object data
    pub object_data: ObjectData,
    /// Map size, expressed in number of tiles
    pub map_size: TilemapSize,
    /// PhysicsBackend in use
    pub physics_backend: PhysicsBackend,
}

/// This event is sent when a tile registered with [register_tiled_custom_tile] is spawned.
/// It should be handled using an observer. See [associated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/user_properties.rs#L79)
#[derive(Event, Clone, Debug)]
pub struct TiledCustomTileCreated {
    /// Spawned tile entity
    pub entity: Entity,
    /// Tiled map type
    pub map_type: TilemapType,
    /// Tiled tile data
    pub tile_data: TileData,
    /// Map size, expressed in number of tiles
    pub map_size: TilemapSize,
    /// Tile size, expressed in pixels
    pub grid_size: TilemapGridSize,
    /// PhysicsBackend in use
    pub physics_backend: PhysicsBackend,
}

#[cfg(feature = "physics")]
impl TiledObjectCreated {
    pub fn spawn_collider(&self, mut commands: Commands, collider_callback: ColliderCallback) {
        self.physics_backend.insert_object_colliders(
            &mut commands,
            self.entity,
            &self.map_type,
            &self.object_data,
            collider_callback,
        )
    }
}

#[cfg(feature = "physics")]
impl TiledCustomTileCreated {
    pub fn spawn_collider(
        &self,
        mut commands: Commands,
        collision_object_names: ObjectNames,
        collider_callback: ColliderCallback,
    ) {
        if let Some(collision) = &self.tile_data.collision {
            self.physics_backend.insert_tile_colliders(
                &mut commands,
                &ObjectNameFilter::from(&collision_object_names),
                self.entity,
                &self.map_type,
                &self.grid_size,
                collision,
                collider_callback,
            )
        }
    }
}
