use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::{ObjectData, TileData};

#[cfg(feature = "rapier")]
use crate::prelude::*;

#[derive(Event, Clone, Debug)]
pub struct TiledObjectCreated {
    pub entity: Entity,
    pub object_data: ObjectData,
    pub map_size: TilemapSize,
}

#[derive(Event, Clone, Debug)]
pub struct TiledCustomTileCreated {
    pub entity: Entity,
    pub tile_data: TileData,
    pub map_size: TilemapSize,
    pub grid_size: TilemapGridSize,
}

#[cfg(feature = "rapier")]
impl TiledObjectCreated {
    pub fn spawn_rapier_collider(
        &self,
        mut commands: Commands,
        collider_callback: ColliderCallback,
    ) {
        insert_object_colliders(
            &mut commands,
            self.entity,
            &self.object_data,
            collider_callback,
        )
    }
}

#[cfg(feature = "rapier")]
impl TiledCustomTileCreated {
    pub fn spawn_rapier_collider(
        &self,
        mut commands: Commands,
        collision_object_names: ObjectNames,
        collider_callback: ColliderCallback,
    ) {
        if let Some(collision) = &self.tile_data.collision {
            insert_tile_colliders(
                &mut commands,
                &ObjectNameFilter::from(&collision_object_names),
                self.entity,
                &self.grid_size,
                collision,
                collider_callback,
            )
        }
    }
}
