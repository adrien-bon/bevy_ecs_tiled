//! Collider management for Tiled maps and worlds.
//!
//! This module defines marker components and events for colliders generated from Tiled maps and objects.
//! It provides types to distinguish between colliders created from tile layers and object layers,
//! as well as utilities for extracting tile data relevant to collider generation.

use crate::{prelude::*, tiled::helpers::tilemap_offset};
use ::geo::BooleanOps;
use bevy::prelude::*;

/// Marker component for collider's source
///
/// Helps to distinguish between colliders created from Tiled objects and those created from Tiled tile layers.
#[derive(Component, Reflect, Copy, PartialEq, Clone, Debug)]
#[reflect(Component, Debug)]
pub enum TiledColliderSource {
    /// Collider is created by a [`tiled::TileLayer`] (ie. a collection of [`tiled::Tile`])
    TilesLayer,
    /// Collider is created by an [`tiled::Object`]
    Object,
}

/// Relationship [`Component`] for the collider of a [`TiledObject`] or [`TiledLayer::Tiles`].
#[derive(Component, Reflect, Copy, PartialEq, Clone, Debug, Deref)]
#[reflect(Component, Debug)]
#[relationship(relationship_target = TiledColliders, allow_self_referential)]
pub struct TiledColliderOf(pub Entity);

/// Relationship target [`Component`] pointing to all the child [`TiledColliderOf`]s (eg. entities holding a physics collider).
#[derive(Component, Reflect, Debug, Deref)]
#[reflect(Component, Debug)]
#[relationship_target(relationship = TiledColliderOf)]
pub struct TiledColliders(Vec<Entity>);

/// Physics collider raw geometry
#[derive(Component, PartialEq, Clone, Debug, Deref)]
#[require(Transform)]
pub struct TiledColliderPolygons(pub Vec<geo::MultiPolygon<f32>>);

/// Event emitted when a collider is created from a Tiled map or world.
///
/// You can determine collider origin using the inner [`TiledColliderSource`] or [`TiledColliderOf`] components.
/// See also [`TiledEvent`]
#[derive(Clone, Copy, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct ColliderCreated {
    /// Origin of the collider
    pub source: TiledColliderSource,
    /// Parent entity of the collider
    pub collider_of: TiledColliderOf,
}

impl ColliderCreated {
    /// Create a new [`ColliderCreated`] event
    pub fn new(source: TiledColliderSource, parent: Entity) -> Self {
        Self {
            source,
            collider_of: TiledColliderOf(parent),
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledColliderSource>();
    app.register_type::<TiledColliderOf>();
    app.register_type::<TiledColliders>();
    app.add_message::<TiledEvent<ColliderCreated>>()
        .register_type::<TiledEvent<ColliderCreated>>();
}

impl<'a> TiledEvent<ColliderCreated> {
    /// Returns a vector containing [`tiled::Tile`]s in this layer as well as their
    /// relative position from their parent [`TiledLayer`] [`Entity`].
    ///
    /// Note that tiles are read from the [`TiledMapAsset`]: modifications made to the
    /// [`TiledTile`] entities after the map has been spawned are not taken into account.
    pub fn tiles(
        &self,
        assets: &'a Assets<TiledMapAsset>,
        anchor: &TilemapAnchor,
    ) -> Vec<(Vec2, tiled::Tile<'a>)> {
        let Some(map_asset) = self.map_asset(assets) else {
            return vec![];
        };
        self.layer(assets)
            .and_then(|layer| layer.as_tile_layer())
            .map(|layer| {
                let mut out = vec![];
                map_asset.for_each_tile(&layer, |layer_tile, _, tile_pos, _| {
                    if let Some(tile) = layer_tile.get_tile() {
                        let tile_coords =
                            map_asset.tile_relative_position(&tile_pos, &tile_size(&tile), anchor);
                        let offset = tilemap_offset(&map_asset.map, tile.tileset());
                        out.push((tile_coords + offset, tile));
                    }
                });
                out
            })
            .unwrap_or_default()
    }
}

/// Compute the collider polygons of a [`TiledObject`], relative to the object itself.
///
/// For a [`TiledObject::Tile`], polygons are computed from the tile collision objects which match `filter`.
pub(crate) fn object_polygons(
    map_asset: &TiledMapAsset,
    tiled_object: &TiledObject,
    filter: &TiledFilter,
) -> Vec<geo::Polygon<f32>> {
    match tiled_object {
        // If the object has a tile, we need to handle its collision data
        TiledObject::Tile {
            width,
            height,
            tileset_id,
            tile_id,
            flip_x,
            flip_y,
        } => {
            let Some(tile) = map_asset.tile(*tileset_id, *tile_id) else {
                return vec![];
            };
            let Some(object_layer_data) = &tile.collision else {
                return vec![];
            };
            let (width, height) = (*width, *height);
            let tile_size = tile_size(&tile);
            let mut scale = Vec2::new(width, height) / Vec2::new(tile_size.x, tile_size.y);
            let mut offset = Vec2::new(
                tile.tileset().offset_x as f32,
                -tile.tileset().offset_y as f32,
            ) * scale;
            if *flip_x {
                scale.x *= -1.;
                offset.x += width;
            }
            if *flip_y {
                scale.y *= -1.;
                offset.y -= height;
            }
            polygons_from_tile(
                map_asset,
                object_layer_data,
                filter,
                &TilemapTileSize::new(width, height),
                offset,
                scale,
            )
        }
        // If the object does not have a tile, we can create a collider directly from itself
        _ => tiled_object
            .polygon(
                &GlobalTransform::default(),
                matches!(
                    tilemap_type_from_map(&map_asset.map),
                    TilemapType::Isometric(..)
                ),
                &map_asset.tilemap_size,
                &grid_size_from_map(&map_asset.map),
                map_asset.tiled_offset,
            )
            .into_iter()
            .collect(),
    }
}

/// Compute the collider polygons of a set of tiles, relative to their parent [`TiledLayer::Tiles`].
///
/// Tiles are given as their position, tileset ID and [`tiled::TileId`].
/// Polygons are computed from the tiles collision objects which match `filter`.
pub(crate) fn tiles_polygons(
    map_asset: &TiledMapAsset,
    anchor: &TilemapAnchor,
    tiles: impl Iterator<Item = (TilePos, u32, tiled::TileId)>,
    filter: &TiledFilter,
) -> Vec<geo::Polygon<f32>> {
    let mut acc = vec![];
    for (tile_pos, tileset_id, tile_id) in tiles {
        let Some(tile) = map_asset.tile(tileset_id, tile_id) else {
            continue;
        };
        let Some(collision) = &tile.collision else {
            continue;
        };
        let tile_size = tile_size(&tile);
        let tile_position = map_asset.tile_relative_position(&tile_pos, &tile_size, anchor)
            + tilemap_offset(&map_asset.map, tile.tileset());
        acc.extend(polygons_from_tile(
            map_asset,
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
    acc
}

/// Spawn colliders for the given polygons using the physics backend from `settings`.
pub(crate) fn spawn_colliders<T: TiledPhysicsBackend>(
    settings: &TiledPhysicsSettings<T>,
    commands: &mut Commands,
    polygons: Vec<geo::Polygon<f32>>,
    collider_created: TiledEvent<ColliderCreated>,
    message_writer: &mut MessageWriter<TiledEvent<ColliderCreated>>,
    disable_geometry_simplification: bool,
) {
    let polygons = polygons
        .into_iter()
        .map(|p| geo::MultiPolygon::new(vec![p]))
        .collect::<Vec<_>>();

    if polygons.is_empty() {
        return;
    }

    // Try to simplify geometry by merging together adjacent polygons
    let polygons = if !disable_geometry_simplification && settings.simplify_geometry {
        let Some(polygons) = simplify_geometry(polygons, |a, b| a.union(&b)) else {
            warn!(
                "Failed to simplify geometry, skipping this polygon (entity {})",
                collider_created.origin
            );
            return;
        };
        vec![polygons]
    } else {
        polygons
    };

    // Actually spawn our collider using provided physics backend
    if let Some(entity) = settings.backend.spawn_colliders(
        commands,
        collider_created.event.source,
        collider_created.event.collider_of.0,
        polygons.to_owned(),
    ) {
        // Attach collider to its parent and insert additional components
        commands.entity(entity).insert((
            collider_created.event.source,
            TiledColliderPolygons(polygons),
            collider_created.event.collider_of,
        ));

        // Patch origin entity and send collider event
        let mut event = collider_created;
        event.origin = entity;
        event.send(commands, message_writer);
    }
}

fn polygons_from_tile(
    map_asset: &TiledMapAsset,
    object_layer_data: &::tiled::ObjectLayerData,
    filter: &TiledFilter,
    tile_size: &TilemapTileSize,
    offset: Vec2,
    scale: Vec2,
) -> Vec<geo::Polygon<f32>> {
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
        if let Some(p) = TiledObject::from_object_data(map_asset, object).polygon(
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
