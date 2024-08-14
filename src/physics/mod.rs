#[cfg(feature = "rapier")]
pub mod rapier;

#[cfg(feature = "avian")]
pub mod avian;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::{ObjectData, ObjectLayerData};

use crate::prelude::*;

/// Load shapes from an object layer as physics colliders.
///
/// By default `bevy_ecs_tiled` will only process object layers
/// named in `collision_layer_names` in `TiledMapSettings`,
/// and tileset collision shapes named in `collision_object_names`.
///
/// Collision layer names are case-insensitive and leading/trailing
/// whitespace is stripped out.
pub fn insert_object_colliders(
    commands: &mut Commands,
    object_entity: Entity,
    object_data: &ObjectData,
    collider_callback: ColliderCallback,
) {
    #[cfg(feature = "rapier")]
    rapier::insert_rapier_colliders_from_shapes(
        commands,
        object_entity,
        None,
        object_data,
        collider_callback,
    );

    #[cfg(feature = "avian")]
    avian::insert_avian_colliders_from_shapes(
        commands,
        object_entity,
        None,
        object_data,
        collider_callback,
    );
}

pub fn insert_tile_colliders(
    commands: &mut Commands,
    collision_object_names: &ObjectNameFilter,
    tile_entity: Entity,
    grid_size: &TilemapGridSize,
    collision: &ObjectLayerData,
    collider_callback: ColliderCallback,
) {
    for object_data in collision.object_data().iter() {
        if collision_object_names.contains(&object_data.name.trim().to_lowercase()) {
            #[cfg(feature = "rapier")]
            rapier::insert_rapier_colliders_from_shapes(
                commands,
                tile_entity,
                Some(grid_size),
                object_data,
                collider_callback,
            );

            #[cfg(feature = "avian")]
            avian::insert_avian_colliders_from_shapes(
                commands,
                tile_entity,
                Some(grid_size),
                object_data,
                collider_callback,
            );
        }
    }
}
