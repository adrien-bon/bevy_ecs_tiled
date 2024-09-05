use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::{ObjectData, TileData};

#[cfg(feature = "physics")]
use crate::prelude::*;

#[derive(Event, Clone, Debug)]
pub struct TiledObjectCreated {
    pub entity: Entity,
    pub map_type: TilemapType,
    pub object_data: ObjectData,
    pub map_size: TilemapSize,
    pub physics_backend: PhysicsBackend,
}

#[derive(Event, Clone, Debug)]
pub struct TiledCustomTileCreated {
    pub entity: Entity,
    pub map_type: TilemapType,
    pub tile_data: TileData,
    pub map_size: TilemapSize,
    pub grid_size: TilemapGridSize,
    pub physics_backend: PhysicsBackend,
}

#[cfg(any(feature = "rapier", feature = "avian"))]
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
