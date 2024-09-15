//! This module handles all things related to physics.
//! 
//! It is only available when the `physics` feature is enabled.
//! 
//! See the [dedicated book section](https://adrien-bon.github.io/bevy_ecs_tiled/guides/physics.html) for more information.

#[cfg(feature = "rapier")]
pub mod rapier;

#[cfg(feature = "avian")]
pub mod avian;

use core::fmt;

use bevy::{ecs::world::Command, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use tiled::{ObjectData, ObjectLayerData};

use crate::prelude::*;

/// This event is sent when a collider is created while we use [PhysicsBackend::Custom].
/// 
/// This event contains an entity which should be extended by the user.
/// 
/// Note that this empty collider is already attached to its parent (either a tile or an object) and has a `Name`.
/// 
/// Example:
/// ```rust,no_run
/// fn handle_colliders_creation_event(
///     mut commands: Commands,
///     mut ev_custom_collider_created: EventReader<CustomColliderCreationEvent>,
/// ) {
///     for ev in ev_custom_collider_created.read() {
///         commands
///             .entity(ev.collider_entity)
///             .insert(MyCustomPhysicsComponent);
///     }
/// }
/// ```
#[derive(Event, Clone, Debug)]
pub struct CustomColliderCreationEvent {
    /// Collider entity to extend.
    pub collider_entity: Entity,
    /// Tiled map type.
    pub map_type: TilemapType,
    /// Tile size, expressed in pixels.
    /// 
    /// If `None`, it means collider is associated to an object.
    /// If `Some`, it means collider is associated to a tile collision object.
    pub grid_size: Option<TilemapGridSize>,
    /// Tiled object data.
    /// 
    /// There can be several objects (hence, several events) when adding colliders to a tile.
    pub object_data: ObjectData,
}

impl Command for CustomColliderCreationEvent {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

/// Physics backend in use.
/// 
/// Determine which kind of physics colliders will be added.
/// 
/// Note that the default value for this settings depends upon which feature flags are enabled:
/// 
/// | Feature flag | Default `PhysicsBackend` |
/// |--------------|--------------------------|
/// | N/A          | `None`                   |
/// | `avian`      | `Avian`                  |
/// | `rapier`     | `Rapier`                 |
/// | `avian` + `rapier` | `Avian`            |
#[derive(Clone, Resource)]
pub enum PhysicsBackend {
    /// Rapier physics backend.
    /// 
    /// The `rapier` feature must be enabled.
    /// 
    /// Uses the `bevy_rapier2d` crate to automatically add physics colliders.
    Rapier,
    /// Avian physics backend.
    /// 
    /// The `avian` feature must be enabled.
    /// 
    /// Uses the `avian2d` crate to automatically add physics colliders.
    Avian,
    /// No physics backend.
    /// 
    /// No collider will be created.
    None,
    /// Custom physics backend.
    /// 
    /// [CustomColliderCreationEvent] will be triggered when adding a new collider.
    /// 
    /// It's up to the user to handle this event and actually extend provided `Entity` to insert its own physics `Component`s.
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
    pub(crate) fn insert_object_colliders(
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
    pub(crate) fn insert_tile_colliders(
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
