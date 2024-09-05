#[cfg(feature = "rapier")]
pub mod rapier;

#[cfg(feature = "avian")]
pub mod avian;

use core::fmt;
use std::sync::Arc;

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use tiled::{ObjectData, ObjectLayerData};

use crate::prelude::*;

/// Trait defining a generic way of handling colliders
/// across different physics backends.
pub trait HandleColliders {
    /// Load shapes from an object layer as physics colliders.
    ///
    /// By default `bevy_ecs_tiled` will only process object layers
    /// named in `collision_layer_names` in `TiledMapSettings`,
    /// and tileset collision shapes named in `collision_object_names`.
    ///
    /// Collision layer names are case-insensitive and leading/trailing
    /// whitespace is stripped out.
    fn insert_colliders_from_shapes<'a>(
        &self,
        commands: &'a mut Commands,
        parent_entity: Entity,
        map_type: &TilemapType,
        grid_size: Option<&TilemapGridSize>,
        object_data: &ObjectData,
    ) -> Option<EntityCommands<'a>>;
}

#[derive(Clone, Resource)]
pub enum PhysicsBackend {
    Rapier,
    Avian,
    None,
    Custom(Arc<Box<dyn HandleColliders + Send + Sync>>),
}

impl fmt::Debug for PhysicsBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhysicsBackend::Rapier => write!(f, "rapier"),
            PhysicsBackend::Avian => write!(f, "avian"),
            PhysicsBackend::None => write!(f, "none"),
            PhysicsBackend::Custom(_) => write!(f, "custom"),
        }
    }
}

impl Default for PhysicsBackend {
    fn default() -> Self {
        #[cfg(not(any(feature = "rapier", feature = "avian")))]
        return PhysicsBackend::None;

        #[cfg(feature = "rapier")]
        return PhysicsBackend::Rapier;

        #[cfg(feature = "avian")]
        return PhysicsBackend::Avian;
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
                    let e = rapier::insert_rapier_colliders_from_shapes(
                        commands,
                        object_entity,
                        map_type,
                        None,
                        object_data,
                    );

                    if e.is_none() {
                        debug!("failed to create rapier colliders from shapes");
                        return None;
                    }

                    e
                }
                #[cfg(not(feature = "rapier"))]
                {
                    panic!("Requested Rapier physics backend but feature is disabled");
                }
            }
            PhysicsBackend::Avian => {
                #[cfg(feature = "avian")]
                {
                    let e = avian::insert_avian_colliders_from_shapes(
                        commands,
                        object_entity,
                        map_type,
                        None,
                        object_data,
                    );

                    if e.is_none() {
                        debug!("failed to create avian colliders from shapes");
                        return;
                    }

                    e
                }
                #[cfg(not(feature = "avian"))]
                {
                    panic!("Requested Avian physics backend but feature is disabled");
                }
            }
            PhysicsBackend::Custom(backend) => {
                let e = backend.insert_colliders_from_shapes(
                    commands,
                    object_entity,
                    map_type,
                    None,
                    object_data,
                );

                e
            }
            PhysicsBackend::None => {
                trace!("No physics backend enabled, skipping inserting object colliders");
                None
            }
        };

        if let Some(mut entity_commands) = e {
            collider_callback(&mut entity_commands);
        }
    }

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
                            let e = rapier::insert_rapier_colliders_from_shapes(
                                commands,
                                tile_entity,
                                map_type,
                                Some(grid_size),
                                object_data,
                            );

                            if e.is_none() {
                                debug!("failed to create rapier colliders from shapes");
                                return;
                            }

                            e
                        }
                        #[cfg(not(feature = "rapier"))]
                        {
                            panic!("Requested Rapier physics backend but feature is disabled");
                        }
                    }
                    PhysicsBackend::Avian => {
                        #[cfg(feature = "avian")]
                        {
                            let e = avian::insert_avian_colliders_from_shapes(
                                commands,
                                tile_entity,
                                map_type,
                                Some(grid_size),
                                object_data,
                            );

                            if e.is_none() {
                                debug!("failed to create avian colliders from shapes");
                                return;
                            }

                            e
                        }
                        #[cfg(not(feature = "avian"))]
                        {
                            panic!("Requested Avian physics backend but feature is disabled");
                        }
                    }
                    PhysicsBackend::Custom(backend) => {
                        let e = backend.insert_colliders_from_shapes(
                            commands,
                            tile_entity,
                            map_type,
                            Some(grid_size),
                            object_data,
                        );

                        e
                    }
                    PhysicsBackend::None => {
                        trace!("No physics backend enabled, skipping inserting tilecolliders");
                        None
                    }
                };

                if let Some(mut entity_commands) = e {
                    collider_callback(&mut entity_commands);
                }
            }
        }
    }
}
