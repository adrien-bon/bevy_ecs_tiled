//! Module that handles colliders
use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::{map::TilemapSize, tiles::TilePos};
use tiled::{ChunkData, Layer, Map, Object, Tile, TileLayer};

/// Marker component for colliders
#[derive(Component)]
#[require(Transform)]
pub struct TiledColliderMarker;

/// Describe the type of the [TiledCollider].
#[derive(Copy, Clone, Debug)]
pub enum TiledCollider {
    TilesLayer {
        /// ID of the layer
        layer_id: usize,
    },
    /// Collider is created by an [Object]
    Object {
        /// ID of the layer containing the [Object].
        layer_id: usize,
        /// ID of the [Object].
        object_id: usize,
    },
}

impl TiledCollider {
    /// Create a new [TiledCollider::Object].
    pub fn from_object(layer_id: usize, object_id: usize) -> Self {
        Self::Object {
            layer_id,
            object_id,
        }
    }

    /// Create a new [TiledCollider::TilesLayer].
    pub fn from_tiles_layer(layer_id: usize) -> Self {
        Self::TilesLayer { layer_id }
    }
}

impl<'a> TiledCollider {
    /// Get the underlying [Layer] of a [TiledCollider].
    pub fn get_layer(&self, map: &'a Map) -> Option<Layer<'a>> {
        match self {
            TiledCollider::Object {
                layer_id,
                object_id: _,
            } => map.get_layer(*layer_id),
            TiledCollider::TilesLayer { layer_id } => map.get_layer(*layer_id),
        }
    }

    /// Get the underlying [Object] of a [TiledCollider].
    pub fn get_object(&self, map: &'a Map) -> Option<Object<'a>> {
        match self {
            TiledCollider::Object {
                layer_id,
                object_id,
            } => map
                .get_layer(*layer_id)
                .and_then(|layer| layer.as_object_layer())
                .and_then(|object_layer| object_layer.get_object(*object_id)),
            _ => None,
        }
    }

    /// Get a vector containing tiles in this layer as well as their relative position to their parent tileset layer.
    pub fn get_tiles(&self, map: &'a Map) -> Vec<(Vec2, Tile<'a>)> {
        match self {
            TiledCollider::TilesLayer { layer_id } => map
                .get_layer(*layer_id)
                .and_then(|layer| layer.as_tile_layer())
                .map(|layer| {
                    let mut out = vec![];
                    match layer {
                        TileLayer::Finite(layer) => {
                            for x in 0..layer.width() {
                                for y in 0..layer.height() {
                                    let mapped_x = x as i32;
                                    let mapped_y = (layer.height() - 1 - y) as i32;
                                    if let Some(tile) = layer.get_tile(mapped_x, mapped_y) {
                                        if let Some(tile) = tile.get_tile() {
                                            let tile_pos = TilePos::new(x, y).center_in_world(
                                                &get_grid_size(map),
                                                &get_map_type(map),
                                            );
                                            out.push((tile_pos, tile));
                                        }
                                    }
                                }
                            }
                        }
                        TileLayer::Infinite(layer) => {
                            let grid_size = get_grid_size(map);
                            let map_type = get_map_type(map);
                            let (topleft_x, topleft_y) =
                                layer.chunks().fold((999999, 999999), |acc, (pos, _)| {
                                    (acc.0.min(pos.0), acc.1.min(pos.1))
                                });
                            let (bottomright_x, bottomright_y) = layer
                                .chunks()
                                .fold((topleft_x, topleft_y), |acc, (pos, _)| {
                                    (acc.0.max(pos.0), acc.1.max(pos.1))
                                });
                            let map_size = TilemapSize {
                                x: (bottomright_x - topleft_x + 1) as u32 * ChunkData::WIDTH,
                                y: (bottomright_y - topleft_y + 1) as u32 * ChunkData::HEIGHT,
                            };
                            let origin = (
                                topleft_x as f32 * ChunkData::WIDTH as f32,
                                ((topleft_y as f32 / 2.) * ChunkData::HEIGHT as f32) + 1.,
                            );
                            for (chunk_pos, chunk) in layer.chunks() {
                                let chunk_pos_mapped =
                                    (chunk_pos.0 - topleft_x, chunk_pos.1 - topleft_y);
                                for x in 0..ChunkData::WIDTH {
                                    for y in 0..ChunkData::HEIGHT {
                                        if let Some(tile) = chunk.get_tile(x as i32, y as i32) {
                                            if let Some(tile) = tile.get_tile() {
                                                let (tile_x, tile_y) = (
                                                    chunk_pos_mapped.0 * ChunkData::WIDTH as i32
                                                        + x as i32,
                                                    chunk_pos_mapped.1 * ChunkData::HEIGHT as i32
                                                        + y as i32,
                                                );
                                                let tile_pos = TilePos {
                                                    x: tile_x as u32,
                                                    y: map_size.y - 1 - tile_y as u32,
                                                };
                                                let mut tile_pos =
                                                    tile_pos.center_in_world(&grid_size, &map_type);
                                                tile_pos += Vec2::new(
                                                    origin.0 * grid_size.x,
                                                    origin.1 * grid_size.y,
                                                );
                                                out.push((tile_pos, tile));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    out
                })
                .unwrap_or_default(),
            _ => vec![],
        }
    }
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

pub(super) fn spawn_colliders<T: super::TiledPhysicsBackend>(
    backend: &T,
    parent: Entity,
    commands: &mut Commands,
    map: &Map,
    names: &TiledName,
    collider: &TiledCollider,
) {
    for spawn_infos in
        backend.spawn_colliders(commands, map, &TiledNameFilter::from(names), collider)
    {
        commands
            .entity(spawn_infos.entity)
            .insert((
                TiledColliderMarker,
                Name::new(format!("Collider: {}", spawn_infos.name)),
                Transform {
                    translation: Vec3::new(spawn_infos.position.x, spawn_infos.position.y, 0.),
                    rotation: Quat::from_rotation_z(f32::to_radians(spawn_infos.rotation)),
                    ..default()
                },
            ))
            .set_parent(parent);
    }
}
