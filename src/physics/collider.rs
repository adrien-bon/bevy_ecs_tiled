use crate::prelude::*;
use bevy::prelude::*;
use tiled::{Layer, Map, Object, ObjectData, Tile};

#[derive(Clone, Debug)]
pub enum TiledColliderSource {
    Object { layer_id: usize, object_id: usize },
    Tile { layer_id: usize, x: i32, y: i32 },
}

impl<'a> TiledColliderSource {
    pub fn new_object(layer_id: usize, object_id: usize) -> Self {
        Self::Object {
            layer_id,
            object_id,
        }
    }

    pub fn new_tile(layer_id: usize, x: i32, y: i32) -> Self {
        Self::Tile { layer_id, x, y }
    }

    pub fn layer(&self, map: &'a Map) -> Layer<'a> {
        match self {
            TiledColliderSource::Tile {
                layer_id,
                x: _,
                y: _,
            } => map.get_layer(*layer_id).unwrap(),
            TiledColliderSource::Object {
                layer_id,
                object_id: _,
            } => map.get_layer(*layer_id).unwrap(),
        }
    }

    pub fn tile(&self, map: &'a Map) -> Option<Tile<'a>> {
        match self {
            TiledColliderSource::Tile { layer_id, x, y } => Some(
                map.get_layer(*layer_id)
                    .unwrap()
                    .as_tile_layer()
                    .unwrap()
                    .get_tile(*x, *y)
                    .unwrap()
                    .get_tile()
                    .unwrap(),
            ),
            _ => None,
        }
    }

    pub fn object(&self, map: &'a Map) -> Option<Object<'a>> {
        match self {
            TiledColliderSource::Object {
                layer_id,
                object_id,
            } => Some(
                map.get_layer(*layer_id)
                    .unwrap()
                    .as_object_layer()
                    .unwrap()
                    .get_object(*object_id)
                    .unwrap(),
            ),
            _ => None,
        }
    }
}

#[derive(Event, Clone, Debug)]
pub struct TiledColliderCreated {
    pub colliders_entities_list: Vec<Entity>,
    pub map_handle: Handle<TiledMap>,
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
    map: &Map,
    collider_source: &TiledColliderSource,
    object_data: &ObjectData,
    parent_entity: Entity,
    offset: Vec2,
) -> Option<Entity> {
    if let Some((position, entity)) =
        backend.spawn_collider(commands, map, collider_source, object_data)
    {
        let transform = Transform {
            translation: Vec3::new(offset.x, offset.y, 0.),
            rotation: Quat::from_rotation_z(f32::to_radians(-object_data.rotation)),
            ..default()
        } * Transform::from_translation(Vec3::new(position.x, position.y, 0.));
        commands
            .entity(entity)
            .insert(TransformBundle::from_transform(transform))
            .insert(Name::new(format!("Collider({})", object_data.name)))
            .set_parent(parent_entity);
        Some(entity)
    } else {
        None
    }
}
