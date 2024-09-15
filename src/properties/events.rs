//! Events related to Tiled custom properties

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::{ObjectData, TileData};

use crate::prelude::*;
/// Entity-scoped event sent when a Tiled object `Entity` is spawned.
/// 
/// Note this event is only sent for `Entity` which have been spawned using the [TiledObject](../prelude/derive.TiledObject.html) derive macros and with the `tiled_observer` attribute set.
/// It should be handled using the observer function provided to this `tiled_observer` attribute.
/// 
/// ```rust,no_run
/// #[derive(TiledObject, Component, Default)]
/// #[tiled_observer(my_observer)]
/// struct ObjectGraphics {
///     color: bevy::color::Color,
///     is_visible: bool,
/// }
/// 
/// // Note this is a standard Bevy observer so it accepts any regular system parameters
/// fn my_observer(trigger: Trigger<TiledObjectCreated>) {
/// // do things here !
/// }
/// ```
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
    /// PhysicsBackend currrently in use
    pub physics_backend: PhysicsBackend,
}

#[cfg(feature = "physics")]
impl TiledObjectCreated {
    /// Automatically spawn physics colliders associated to this object
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

/// Entity-scoped event sent when a Tiled custom tile `Entity` is spawned.
/// 
/// Note this event is only sent for `Entity` which have been spawned using the [TiledCustomTile](../prelude/derive.TiledCustomTile.html) derive macros and with the `tiled_observer` attribute set.
/// It should be handled using the observer function provided to this `tiled_observer` attribute.
/// 
/// ```rust,no_run
/// #[derive(TiledCustomTile, Component, Default)]
/// #[tiled_observer(my_observer)]
/// struct TileMovement {
///     movement_cost: i32,
///     has_road: bool,
/// }
/// 
/// // Note this is a standard Bevy observer so it accepts any regular system parameters
/// fn my_observer(trigger: Trigger<TiledCustomTileCreated>) {
/// // do things here !
/// }
/// ```
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
impl TiledCustomTileCreated {
    /// Automatically spawn physics colliders associated to this tile
    /// 
    /// Note you must provide a custom [ObjectNames] filter to select which objects your want to add colliders for.
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
