//! Collider management for Tiled maps and worlds.
//!
//! This module defines marker components and events for colliders generated from Tiled maps and objects.
//! It provides types to distinguish between colliders created from tile layers and object layers,
//! as well as utilities for extracting tile data relevant to collider generation.

use crate::{prelude::*, tiled::helpers::tilemap_offset};
use ::geo::{BooleanOps, MapCoords};
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

/// Apply a [`TileFlip`] to a coordinate local to a tile of the given size.
///
/// Coordinates use Bevy referential: origin is the tile bottom-left corner and Y axis points up.
/// Flips are applied the same way `bevy_ecs_tilemap` renders them (and Tiled draws them):
/// diagonal flip first (ie. swapping X and Y axis in image space), then horizontal and vertical flips.
fn flip_tile_coord(coord: geo::Coord<f32>, size: Vec2, flip: &TileFlip) -> geo::Coord<f32> {
    // Normalized coordinates in image space: origin is top-left corner and Y axis points down
    let (mut u, mut v) = (coord.x / size.x, 1. - coord.y / size.y);
    if flip.d {
        std::mem::swap(&mut u, &mut v);
    }
    if flip.x {
        u = 1. - u;
    }
    if flip.y {
        v = 1. - v;
    }
    geo::Coord {
        x: u * size.x,
        y: (1. - v) * size.y,
    }
}

/// Compute the collider polygons of a set of tiles, relative to their parent [`TiledLayer::Tiles`].
///
/// Tiles are given as their position, tileset ID, [`tiled::TileId`] and [`TileFlip`].
/// Polygons are computed from the tiles collision objects which match `filter`.
pub(crate) fn tiles_polygons(
    map_asset: &TiledMapAsset,
    anchor: &TilemapAnchor,
    tiles: impl Iterator<Item = (TilePos, u32, tiled::TileId, TileFlip)>,
    filter: &TiledFilter,
) -> Vec<geo::Polygon<f32>> {
    let mut acc = vec![];
    for (tile_pos, tileset_id, tile_id, flip) in tiles {
        let Some(tile) = map_asset.tile(tileset_id, tile_id) else {
            continue;
        };
        let Some(collision) = &tile.collision else {
            continue;
        };
        let tile_size = tile_size(&tile);
        let tile_position = map_asset.tile_relative_position(&tile_pos, &tile_size, anchor)
            + tilemap_offset(&map_asset.map, tile.tileset());
        let size = Vec2::new(tile_size.x, tile_size.y);
        let bottom_left = tile_position - size / 2.;
        let flipped = flip.x || flip.y || flip.d;
        // Compute polygons relative to the tile bottom-left corner, apply
        // tile flips then move them to the actual tile position
        acc.extend(
            polygons_from_tile(
                map_asset,
                collision,
                filter,
                &tile_size,
                Vec2::ZERO,
                Vec2::ONE,
            )
            .into_iter()
            .map(|polygon| {
                polygon.map_coords(|coord| {
                    let coord = if flipped {
                        flip_tile_coord(coord, size, &flip)
                    } else {
                        coord
                    };
                    geo::Coord {
                        x: coord.x + bottom_left.x,
                        y: coord.y + bottom_left.y,
                    }
                })
            }),
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    fn flip(x: bool, y: bool, d: bool) -> TileFlip {
        TileFlip { x, y, d }
    }

    fn apply(x: f32, y: f32, size: Vec2, f: TileFlip) -> (f32, f32) {
        let c = flip_tile_coord(geo::Coord { x, y }, size, &f);
        (c.x, c.y)
    }

    #[test]
    fn flip_square_tile() {
        let size = Vec2::new(16., 16.);
        // Point near the tile top-left corner
        assert_eq!(apply(2., 14., size, flip(false, false, false)), (2., 14.));
        assert_eq!(apply(2., 14., size, flip(true, false, false)), (14., 14.));
        assert_eq!(apply(2., 14., size, flip(false, true, false)), (2., 2.));
        assert_eq!(apply(2., 14., size, flip(true, true, false)), (14., 2.));
        // This point is on the image diagonal: diagonal flip does not move it
        assert_eq!(apply(2., 14., size, flip(false, false, true)), (2., 14.));
        // Diagonal flip swaps X and Y axis in image space (Y pointing down)
        assert_eq!(apply(2., 10., size, flip(false, false, true)), (6., 14.));
        // Diagonal flip is applied before horizontal and vertical ones
        assert_eq!(apply(2., 10., size, flip(true, false, true)), (10., 14.));
        assert_eq!(apply(2., 10., size, flip(false, true, true)), (6., 2.));
    }

    #[test]
    fn flip_non_square_tile() {
        let size = Vec2::new(16., 8.);
        assert_eq!(apply(4., 2., size, flip(true, false, false)), (12., 2.));
        assert_eq!(apply(4., 2., size, flip(false, true, false)), (4., 6.));
        // Coordinates are normalized: diagonal flip stretches the tile like rendering does
        assert_eq!(apply(4., 2., size, flip(false, false, true)), (12., 6.));
    }
}
