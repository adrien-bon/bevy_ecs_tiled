#[cfg(feature = "rapier")]
pub mod rapier;

#[cfg(feature = "avian")]
pub mod avian;

use core::fmt;

use bevy::{ecs::world::Command, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use tiled::{ObjectData, ObjectLayerData};

use crate::prelude::*;

#[derive(Event, Clone, Debug)]
pub struct CustomColliderCreationEvent {
    pub collider_entity: Entity,
    pub map_type: TilemapType,
    pub grid_size: Option<TilemapGridSize>,
    pub object_data: ObjectData,
}

impl Command for CustomColliderCreationEvent {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

#[derive(Clone, Resource)]
pub enum PhysicsBackend {
    Rapier,
    Avian,
    None,
    Custom,
}

impl fmt::Debug for PhysicsBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhysicsBackend::Rapier => write!(f, "rapier"),
            PhysicsBackend::Avian => write!(f, "avian"),
            PhysicsBackend::None => write!(f, "none"),
            PhysicsBackend::Custom => write!(f, "custom"),
        }
    }
}

impl Default for PhysicsBackend {
    fn default() -> Self {
        #[cfg(not(any(feature = "rapier", feature = "avian")))]
        return PhysicsBackend::None;

        #[cfg(all(feature = "rapier", feature = "avian"))]
        return PhysicsBackend::Avian;

        #[cfg(all(feature = "avian", not(feature = "rapier")))]
        return PhysicsBackend::Avian;

        #[cfg(all(feature = "rapier", not(feature = "avian")))]
        return PhysicsBackend::Rapier;
    }
}

impl PhysicsBackend {
    /// Load shapes from an object layer as physics colliders.
    ///
    /// By default `bevy_ecs_tiled` will only process object layers
    /// named in `collision_layer_names` in `TiledMapSettings`,
    /// and tileset collision shapes named in `collision_object_names`.
    ///
    /// Collision layer names are case-insensitive and leading/trailing
    /// whitespace is stripped out.
    pub fn insert_object_colliders(
        &self,
        commands: &mut Commands,
        object_entity: Entity,
        map_type: &TilemapType,
        object_data: &ObjectData,
        collider_callback: ColliderCallback,
    ) {
        let e = match self {
            PhysicsBackend::Rapier => {
                #[cfg(feature = "rapier")]
                {
                    rapier::insert_rapier_colliders_from_shapes(
                        commands,
                        object_entity,
                        map_type,
                        None,
                        object_data,
                    )
                }
                #[cfg(not(feature = "rapier"))]
                {
                    panic!("Requested Rapier physics backend but feature is disabled");
                }
            }
            PhysicsBackend::Avian => {
                #[cfg(feature = "avian")]
                {
                    avian::insert_avian_colliders_from_shapes(
                        commands,
                        object_entity,
                        map_type,
                        None,
                        object_data,
                    )
                }
                #[cfg(not(feature = "avian"))]
                {
                    panic!("Requested Avian physics backend but feature is disabled");
                }
            }
            PhysicsBackend::Custom => {
                // Spawn an 'empty collider'
                let collider_entity = commands
                    .spawn_empty()
                    .insert(Name::new(format!("Collider({})", object_data.name)))
                    .set_parent(object_entity)
                    .id();

                // Send event so the user can add its own components
                commands.add(CustomColliderCreationEvent {
                    collider_entity,
                    map_type: *map_type,
                    grid_size: None,
                    object_data: object_data.clone(),
                });

                // Do not return EntityCommands: we don't want to trigger the callback
                None
            }
            PhysicsBackend::None => {
                trace!("No physics backend enabled, skipping inserting object colliders");
                return;
            }
        };

        if let Some(mut entity_commands) = e {
            collider_callback(&mut entity_commands);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_tile_colliders(
        &self,
        commands: &mut Commands,
        collision_object_names: &ObjectNameFilter,
        tile_entity: Entity,
        map_type: &TilemapType,
        grid_size: &TilemapGridSize,
        collision: &ObjectLayerData,
        collider_callback: ColliderCallback,
    ) {
        for object_data in collision.object_data().iter() {
            if collision_object_names.contains(&object_data.name.trim().to_lowercase()) {
                let e = match self {
                    PhysicsBackend::Rapier => {
                        #[cfg(feature = "rapier")]
                        {
                            rapier::insert_rapier_colliders_from_shapes(
                                commands,
                                tile_entity,
                                map_type,
                                Some(grid_size),
                                object_data,
                            )
                        }
                        #[cfg(not(feature = "rapier"))]
                        {
                            panic!("Requested Rapier physics backend but feature is disabled");
                        }
                    }
                    PhysicsBackend::Avian => {
                        #[cfg(feature = "avian")]
                        {
                            avian::insert_avian_colliders_from_shapes(
                                commands,
                                tile_entity,
                                map_type,
                                Some(grid_size),
                                object_data,
                            )
                        }
                        #[cfg(not(feature = "avian"))]
                        {
                            panic!("Requested Avian physics backend but feature is disabled");
                        }
                    }
                    PhysicsBackend::Custom => {
                        // Spawn an 'empty collider'
                        let collider_entity = commands
                            .spawn_empty()
                            .insert(Name::new(format!("Collider({})", object_data.name)))
                            .set_parent(tile_entity)
                            .id();

                        // Send event so the user can add its own components
                        commands.add(CustomColliderCreationEvent {
                            collider_entity,
                            map_type: *map_type,
                            grid_size: Some(*grid_size),
                            object_data: object_data.clone(),
                        });

                        // Do not return EntityCommands: we don't want to trigger the callback
                        None
                    }
                    PhysicsBackend::None => {
                        trace!("No physics backend enabled, skipping inserting tilecolliders");
                        return;
                    }
                };

                if let Some(mut entity_commands) = e {
                    collider_callback(&mut entity_commands);
                }
            }
        }
    }
}
