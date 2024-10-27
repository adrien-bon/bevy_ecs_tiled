//! Module that handles colliders
use crate::prelude::*;
use bevy::prelude::*;
use tiled::{Layer, Map, Object, Tile};

/// Marker component for colliders
#[derive(Component)]
pub struct TiledColliderMarker;

/// Describe the type of the [TiledColliderSource].
#[derive(Copy, Clone, Debug)]
pub enum TiledColliderSourceType {
    /// Collider is created by an [Object]
    Object {
        /// ID of the layer containing the [Object].
        layer_id: usize,
        /// ID of the [Object].
        object_id: usize,
    },
    /// Collider is created by a collider object on a [Tile]
    ///
    /// Note that a [Tile] can have several collider object.
    Tile {
        /// ID of the layer containing the [Tile].
        layer_id: usize,
        /// X position of the [Tile] in Tiled referential.
        x: i32,
        /// Y position of the [Tile] in Tiled referential.
        y: i32,
        /// ID of the collider [Object] for this [Tile].
        ///
        /// ID is unique for a given [Tile].
        object_id: usize,
    },
}

impl TiledColliderSourceType {
    /// Create a new [TiledColliderSourceType] for an [Object].
    pub fn new_object(layer_id: usize, object_id: usize) -> Self {
        Self::Object {
            layer_id,
            object_id,
        }
    }

    /// Create a new [TiledColliderSourceType] for a [Tile].
    pub fn new_tile(layer_id: usize, x: i32, y: i32, object_id: usize) -> Self {
        Self::Tile {
            layer_id,
            x,
            y,
            object_id,
        }
    }
}

/// Describe what is creating a collider.
#[derive(Copy, Clone, Debug)]
pub struct TiledColliderSource {
    /// Parent [Entity] of the collider.
    pub entity: Entity,
    /// Which type of source creates this collider.
    pub ty: TiledColliderSourceType,
}

impl<'a> TiledColliderSource {
    /// Get the underlying [Layer] of a [TiledColliderSource].
    pub fn layer(&self, map: &'a Map) -> Option<Layer<'a>> {
        match self.ty {
            TiledColliderSourceType::Tile {
                layer_id,
                x: _,
                y: _,
                object_id: _,
            } => map.get_layer(layer_id),
            TiledColliderSourceType::Object {
                layer_id,
                object_id: _,
            } => map.get_layer(layer_id),
        }
    }

    /// Get the underlying [Tile] of a [TiledColliderSource].
    pub fn tile(&self, map: &'a Map) -> Option<Tile<'a>> {
        match self.ty {
            TiledColliderSourceType::Tile {
                layer_id,
                x,
                y,
                object_id: _,
            } => map
                .get_layer(layer_id)
                .and_then(|layer| layer.as_tile_layer())
                .and_then(|tile_layer| tile_layer.get_tile(x, y))
                .and_then(|layer_tile| layer_tile.get_tile()),
            _ => None,
        }
    }

    /// Get the underlying [Object] of a [TiledColliderSource].
    pub fn object(&self, map: &'a Map) -> Option<Object<'a>> {
        match self.ty {
            TiledColliderSourceType::Object {
                layer_id,
                object_id,
            } => map
                .get_layer(layer_id)
                .and_then(|layer| layer.as_object_layer())
                .and_then(|object_layer| object_layer.get_object(object_id)),
            _ => None,
        }
    }

    // TODO: we should use this function when I figure out how to prevent cloning ObjectData
    // pub fn object_data(&self, map: &'a Map) -> Option<ObjectData> {
    //     match self {
    //         TiledColliderSourceType::Tile {
    //             layer_id: _,
    //             x: _,
    //             y: _,
    //             object_id,
    //         } => self
    //             .tile(map)
    //             .as_ref()
    //             .and_then(|tile| tile.collision.as_ref())
    //             .map(|collision| collision.object_data())
    //             .and_then(|objects| objects.get(*object_id))
    //             .cloned(),
    //         TiledColliderSourceType::Object {
    //             layer_id: _,
    //             object_id: _,
    //         } => self.object(map).map(|object| object.deref().clone()),
    //     }
    // }
}

/// Spawn informations about a collider
#[derive(Clone, Debug)]
pub struct TiledColliderSpawnInfos {
    /// Name of the collider.
    pub name: String,
    /// [Entity] of the spawned collider.
    pub entity: Entity,
    /// Relative position of the collider from its parent [Entity].
    pub position: Vec2,
    /// Relative rotation of the collider from its parent [Entity].
    pub rotation: f32,
}

/// Event fired when a collider is spawned.
#[derive(Event, Clone, Debug)]
pub struct TiledColliderCreated {
    /// [Handle] to the [TiledMap].
    pub map_handle: Handle<TiledMap>,
    /// Collider spawn informations.
    pub collider: TiledColliderSpawnInfos,
    /// Collider source informations.
    pub collider_source: TiledColliderSource,
}

impl<'a> TiledColliderCreated {
    /// Retrieve the [Map] associated to this [TiledColliderCreated] event.
    pub fn map(&self, map_asset: &'a Res<Assets<TiledMap>>) -> &'a Map {
        &map_asset.get(self.map_handle.id()).unwrap().map
    }

    /// Retrieve the [Layer] associated to this [TiledColliderCreated] event.
    pub fn layer(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Option<Layer<'a>> {
        self.collider_source.layer(self.map(map_asset))
    }

    /// Retrieve the [Tile] associated to this [TiledColliderCreated] event.
    pub fn tile(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Option<Tile<'a>> {
        self.collider_source.tile(self.map(map_asset))
    }

    /// Retrieve the [Object] associated to this [TiledColliderCreated] event.
    pub fn object(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Option<Object<'a>> {
        self.collider_source.object(self.map(map_asset))
    }
}

pub(super) fn spawn_collider<T: super::TiledPhysicsBackend>(
    backend: &T,
    commands: &mut Commands,
    map_asset: &Res<Assets<TiledMap>>,
    map_handle: &Handle<TiledMap>,
    collider_source: &TiledColliderSource,
    offset: Vec2,
) {
    if let Some(tiled_map) = map_asset.get(map_handle) {
        if let Some(collider) = backend.spawn_collider(commands, &tiled_map.map, collider_source) {
            let transform = Transform {
                translation: Vec3::new(offset.x, offset.y, 0.),
                rotation: Quat::from_rotation_z(f32::to_radians(collider.rotation)),
                ..default()
            } * Transform::from_translation(Vec3::new(
                collider.position.x,
                collider.position.y,
                0.,
            ));
            commands
                .entity(collider.entity)
                .insert(TiledColliderMarker)
                .insert(TransformBundle::from_transform(transform))
                .insert(Name::new(format!("Collider: {}", collider.name)))
                .set_parent(collider_source.entity);
            commands.trigger(TiledColliderCreated {
                map_handle: map_handle.clone(),
                collider,
                collider_source: *collider_source,
            });
        }
    }
}
