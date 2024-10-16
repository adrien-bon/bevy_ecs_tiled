use crate::prelude::*;
use bevy::prelude::*;
use tiled::{Layer, Map, Object, Tile};

#[derive(Copy, Clone, Debug)]
pub enum TiledColliderSource {
    Object {
        layer_id: usize,
        object_id: usize,
    },
    Tile {
        layer_id: usize,
        x: i32,
        y: i32,
        object_id: usize,
    },
}

impl<'a> TiledColliderSource {
    pub fn new_object(layer_id: usize, object_id: usize) -> Self {
        Self::Object {
            layer_id,
            object_id,
        }
    }

    pub fn new_tile(layer_id: usize, x: i32, y: i32, object_id: usize) -> Self {
        Self::Tile {
            layer_id,
            x,
            y,
            object_id,
        }
    }

    pub fn layer(&self, map: &'a Map) -> Layer<'a> {
        match self {
            TiledColliderSource::Tile {
                layer_id,
                x: _,
                y: _,
                object_id: _,
            } => map.get_layer(*layer_id).unwrap(),
            TiledColliderSource::Object {
                layer_id,
                object_id: _,
            } => map.get_layer(*layer_id).unwrap(),
        }
    }

    pub fn tile(&self, map: &'a Map) -> Option<Tile<'a>> {
        match self {
            TiledColliderSource::Tile {
                layer_id,
                x,
                y,
                object_id: _,
            } => map
                .get_layer(*layer_id)
                .and_then(|layer| layer.as_tile_layer())
                .and_then(|tile_layer| tile_layer.get_tile(*x, *y))
                .and_then(|layer_tile| layer_tile.get_tile()),
            _ => None,
        }
    }

    pub fn object(&self, map: &'a Map) -> Option<Object<'a>> {
        match self {
            TiledColliderSource::Object {
                layer_id,
                object_id,
            } => map
                .get_layer(*layer_id)
                .and_then(|layer| layer.as_object_layer())
                .and_then(|object_layer| object_layer.get_object(*object_id)),
            _ => None,
        }
    }

    // TODO: we should use this function when I figure out how to prevent cloning ObjectData
    // pub fn object_data(&self, map: &'a Map) -> Option<ObjectData> {
    //     match self {
    //         TiledColliderSource::Tile {
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
    //         TiledColliderSource::Object {
    //             layer_id: _,
    //             object_id: _,
    //         } => self.object(map).map(|object| object.deref().clone()),
    //     }
    // }
}

#[derive(Clone, Debug)]
pub struct TiledColliderSpawnInfos {
    pub name: String,
    pub entity: Entity,
    pub position: Vec2,
    pub rotation: f32,
}

#[derive(Event, Clone, Debug)]
pub struct TiledColliderCreated {
    pub map_handle: Handle<TiledMap>,
    pub collider: Entity,
    pub collider_source: TiledColliderSource,
    pub collider_source_entity: Entity,
}

impl<'a> TiledColliderCreated {
    /// Retrieve the [Map] associated to this [TiledColliderCreated] event.
    pub fn map(&self, map_asset: &'a Res<Assets<TiledMap>>) -> &'a Map {
        &map_asset.get(self.map_handle.id()).unwrap().map
    }

    /// Retrieve the [Layer] associated to this [TiledColliderCreated] event.
    pub fn layer(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Layer<'a> {
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
    parent_entity: Entity,
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
                .insert(TransformBundle::from_transform(transform))
                .insert(Name::new(format!("Collider: {}", collider.name)))
                .set_parent(parent_entity);
            commands.trigger(TiledColliderCreated {
                map_handle: map_handle.clone(),
                collider: collider.entity,
                collider_source: *collider_source,
                collider_source_entity: parent_entity,
            });
        }
    }
}
