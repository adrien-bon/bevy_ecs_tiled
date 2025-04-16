//! Module that handles colliders
use crate::prelude::*;
use bevy::prelude::*;
use tiled::{Layer, Object, Tile};

/// Marker component for colliders
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Transform)]
pub struct TiledColliderMarker;

/// Describe the type of the [TiledCollider].
#[derive(Copy, Clone, Debug)]
pub enum TiledCollider {
    /// Collider is created by a tiles [Layer]
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
    pub fn get_layer(&self, tiled_map: &'a TiledMap) -> Option<Layer<'a>> {
        match self {
            TiledCollider::Object {
                layer_id,
                object_id: _,
            } => tiled_map.map.get_layer(*layer_id),
            TiledCollider::TilesLayer { layer_id } => tiled_map.map.get_layer(*layer_id),
        }
    }

    /// Get the underlying [Object] of a [TiledCollider].
    pub fn get_object(&self, tiled_map: &'a TiledMap) -> Option<Object<'a>> {
        match self {
            TiledCollider::Object {
                layer_id,
                object_id,
            } => tiled_map
                .map
                .get_layer(*layer_id)
                .and_then(|layer| layer.as_object_layer())
                .and_then(|object_layer| object_layer.get_object(*object_id)),
            _ => None,
        }
    }

    /// Get a vector containing tiles in this layer as well as their relative position to their parent tileset layer.
    pub fn get_tiles(
        &self,
        tiled_map: &'a TiledMap,
        anchor: &TilemapAnchor,
    ) -> Vec<(Vec2, Tile<'a>)> {
        match self {
            TiledCollider::TilesLayer { layer_id } => tiled_map
                .map
                .get_layer(*layer_id)
                .and_then(|layer| layer.as_tile_layer())
                .map(|layer| {
                    let mut out = vec![];
                    for_each_tile(tiled_map, &layer, |layer_tile, _, tile_pos, _| {
                        if let Some(tile) = layer_tile.get_tile() {
                            let grid_size = get_grid_size(&tiled_map.map);
                            let tile_size = tile_size_from_grid(&grid_size);
                            let tile_coords = tile_pos.center_in_world(
                                &tiled_map.tilemap_size,
                                &grid_size,
                                &tile_size,
                                &get_map_type(&tiled_map.map),
                                anchor,
                            );
                            out.push((tile_coords, tile));
                        }
                    });
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
    /// Relative position and rotation of the collider from its parent [Entity].
    pub transform: Transform,
}

pub(super) fn spawn_colliders<T: super::TiledPhysicsBackend>(
    backend: &T,
    parent: Entity,
    commands: &mut Commands,
    tiled_map: &TiledMap,
    names: &TiledName,
    collider: &TiledCollider,
    anchor: &TilemapAnchor,
) {
    for spawn_infos in backend.spawn_colliders(
        commands,
        tiled_map,
        &TiledNameFilter::from(names),
        collider,
        anchor,
    ) {
        commands.entity(spawn_infos.entity).insert((
            TiledColliderMarker,
            Name::new(format!("Collider: {}", spawn_infos.name)),
            ChildOf(parent),
            spawn_infos.transform,
        ));
    }
}
