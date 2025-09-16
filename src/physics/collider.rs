//! Collider management for Tiled maps and worlds.
//!
//! This module defines marker components and events for colliders generated from Tiled maps and objects.
//! It provides types to distinguish between colliders created from tile layers and object layers,
//! as well as utilities for extracting tile data relevant to collider generation.

use std::collections::VecDeque;

use crate::prelude::*;
use bevy::prelude::*;
use geo::BooleanOps;
use tiled::{ObjectLayerData, ObjectShape};

/// Marker component for collider's origin
///
/// Helps to distinguish between colliders created from Tiled objects and those created from Tiled tile layers.
#[derive(Component, Reflect, Copy, PartialEq, Clone, Debug)]
#[reflect(Component, Debug)]
pub enum TiledColliderOrigin {
    /// Collider is created by a [`tiled::TileLayer`] (ie. a collection of [`Tile`])
    TilesLayer,
    /// Collider is created by an [`tiled::Object`]
    Object,
}

/// Relationship [`Component`] for the collider of a [`TiledObject`] or [`TiledLayer::Tiles`].
#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Debug)]
#[relationship(relationship_target = TiledColliders)]
pub struct TiledColliderOf(pub Entity);

/// Relationship target [`Component`] pointing to all the child [`TiledColliderOf`]s (eg. entities holding a physics collider).
#[derive(Component, Reflect, Debug)]
#[reflect(Component, Debug)]
#[relationship_target(relationship = TiledColliderOf)]
pub struct TiledColliders(Vec<Entity>);

/// Collider raw geometry
#[derive(Component, PartialEq, Clone, Debug, Deref)]
#[require(Transform)]
pub struct TiledColliderPolygons(pub MultiPolygon<f32>);

/// Event emitted when a collider is created from a Tiled map or world.
///
/// You can determine collider origin using the inner [`TiledColliderOrigin`] or [`TiledColliderOf`] components.
/// See also [`TiledEvent`]
#[derive(Clone, Copy, PartialEq, Debug, Reflect, Deref)]
#[reflect(Clone, PartialEq)]
pub struct ColliderCreated(pub TiledColliderOrigin);

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledColliderOrigin>();
    app.register_type::<TiledColliderOf>();
    app.register_type::<TiledColliders>();
    app.add_event::<TiledEvent<ColliderCreated>>()
        .register_type::<TiledEvent<ColliderCreated>>();
}

impl<'a> TiledEvent<ColliderCreated> {
    /// Returns a vector containing [`Tile`]s in this layer as well as their
    /// relative position from their parent [`crate::tiled::tile::TiledTilemap`] [`Entity``].
    pub fn get_tiles(
        &self,
        assets: &'a Res<Assets<TiledMapAsset>>,
        anchor: &TilemapAnchor,
    ) -> Vec<(Vec2, Tile<'a>)> {
        let Some(map_asset) = self.get_map_asset(assets) else {
            return vec![];
        };
        self.get_layer(assets)
            .and_then(|layer| layer.as_tile_layer())
            .map(|layer| {
                let mut out = vec![];
                map_asset.for_each_tile(&layer, |layer_tile, _, tile_pos, _| {
                    if let Some(tile) = layer_tile.get_tile() {
                        let tile_coords =
                            map_asset.tile_relative_position(&tile_pos, &tile_size(&tile), anchor);
                        let offset = Vec2::new(
                            tile.tileset().offset_x as f32,
                            -tile.tileset().offset_y as f32,
                        );
                        out.push((tile_coords + offset, tile));
                    }
                });
                out
            })
            .unwrap_or_default()
    }
}

pub(crate) fn spawn_colliders<T: TiledPhysicsBackend>(
    backend: &T,
    commands: &mut Commands,
    assets: &Res<Assets<TiledMapAsset>>,
    anchor: &TilemapAnchor,
    filter: &TiledFilter,
    source: TiledEvent<ColliderCreated>,
    parent: Entity,
    event_writer: &mut EventWriter<TiledEvent<ColliderCreated>>,
) {
    let Some(map_asset) = source.get_map_asset(assets) else {
        return;
    };

    let polygons = match *source.event {
        TiledColliderOrigin::Object => {
            if let Some(object) = source.get_object(assets) {
                match object.get_tile() {
                    // If the object does not have a tile, we can create a collider directly from itself
                    None => {
                        let global_transform = &GlobalTransform::default();
                        TiledObject::from_object_data(&object)
                            .polygon(
                                global_transform,
                                matches!(
                                    tilemap_type_from_map(&map_asset.map),
                                    TilemapType::Isometric(..)
                                ),
                                &map_asset.tilemap_size,
                                &grid_size_from_map(&map_asset.map),
                                map_asset.tiled_offset,
                            )
                            .map(|p| vec![p])
                    }
                    // If the object has a tile, we need to handle its collision data
                    Some(object_tile) => object_tile.get_tile().map(|tile| {
                        let Some(object_layer_data) = &tile.collision else {
                            return vec![];
                        };
                        let ObjectShape::Rect { width, height } = object.shape else {
                            return vec![];
                        };

                        let tile_size = tile_size(&tile);
                        let mut scale =
                            Vec2::new(width, height) / Vec2::new(tile_size.x, tile_size.y);
                        let mut offset = Vec2::new(
                            tile.tileset().offset_x as f32,
                            -tile.tileset().offset_y as f32,
                        ) * scale;
                        if object_tile.flip_h {
                            scale.x *= -1.;
                            offset.x += width;
                        }
                        if object_tile.flip_v {
                            scale.y *= -1.;
                            offset.y -= height;
                        }
                        polygons_from_tile(
                            object_layer_data,
                            filter,
                            &TilemapTileSize::new(width, height),
                            offset,
                            scale,
                        )
                    }),
                }
                .unwrap_or_default()
            } else {
                vec![]
            }
        }
        TiledColliderOrigin::TilesLayer => {
            let mut acc = vec![];

            // Iterate over all tiles in the layer and create colliders for each
            for (tile_position, tile) in source.get_tiles(assets, anchor) {
                if let Some(collision) = &tile.collision {
                    let tile_size = tile_size(&tile);
                    acc.extend(polygons_from_tile(
                        collision,
                        filter,
                        &tile_size,
                        Vec2::new(
                            tile_position.x - tile_size.x / 2.,
                            tile_position.y - tile_size.y / 2.,
                        ),
                        Vec2::ONE,
                    ));
                }
            }
            acc
        }
    }
    .into_iter()
    .map(|p| MultiPolygon::new(vec![p]))
    .collect::<Vec<_>>();

    // Try to simplify geometry: merge together adjacent polygons
    let Some(polygons) = divide_reduce(polygons, |a, b| a.union(&b)) else {
        return;
    };

    // Actually spawn our colliders using provided physics backend
    for entity in backend.spawn_colliders(commands, &source, &polygons) {
        // Attach collider to its parent and insert additional components
        commands.entity(entity).insert((
            *source.event,
            TiledColliderPolygons(polygons.to_owned()),
            ChildOf(parent),
            TiledColliderOf(parent),
        ));
        // Patch origin entity and send collider event
        let mut event = source;
        event.origin = entity;
        event.send(commands, event_writer);
    }
}

fn polygons_from_tile(
    object_layer_data: &ObjectLayerData,
    filter: &TiledFilter,
    tile_size: &TilemapTileSize,
    offset: Vec2,
    scale: Vec2,
) -> Vec<GeoPolygon<f32>> {
    let mut polygons = vec![];
    for object in object_layer_data.object_data() {
        if !filter.matches(&object.name) {
            continue;
        }

        let pos = offset + Vec2::new(object.x * scale.x, tile_size.y - object.y * scale.y);
        let transform = GlobalTransform::from_isometry(Isometry3d {
            rotation: Quat::from_rotation_z(f32::to_radians(-object.rotation)),
            translation: pos.extend(0.).into(),
        }) * Transform::from_scale(scale.extend(1.));

        // Special case for tiles: our referential is local to the tile
        // do not use TilemapSize and TilemapGridSize relative to the whole map
        if let Some(p) = TiledObject::from_object_data(object).polygon(
            &transform,
            false, // we do not support 'isometric' tilesets
            &TilemapSize::new(1, 1),
            &TilemapGridSize::new(tile_size.x, tile_size.y),
            Vec2::ZERO,
        ) {
            polygons.push(p);
        }
    }
    polygons
}

fn divide_reduce<T>(list: Vec<T>, mut reduction: impl FnMut(T, T) -> T) -> Option<T> {
    let mut queue = VecDeque::from(list);

    while queue.len() > 1 {
        for _ in 0..(queue.len() / 2) {
            let (one, two) = (queue.pop_front().unwrap(), queue.pop_front().unwrap());
            queue.push_back(reduction(one, two));
        }
    }

    queue.pop_back()
}
