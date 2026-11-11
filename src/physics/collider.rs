//! Collider management for Tiled maps and worlds.
//!
//! This module defines marker components and events for colliders generated from Tiled maps and objects.
//! It provides types to distinguish between colliders created from tile layers and object layers.
//!
//! Colliders are computed from tiles and objects entities, and spawned again when these entities change.

use crate::{prelude::*, tiled::helpers::tilemap_offset};
use ::geo::{MapCoords, Orient};
use bevy::{platform::collections::HashMap, prelude::*};

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

/// How physics colliders are created from the geometry of Tiled objects and tiles.
///
/// The strategy used for a whole map is given by [`TiledPhysicsSettings::collider_strategy`].
///
/// This enum is also a [`Component`], which can be inserted on a given [`TiledLayer`] or [`TiledObject`]
/// entity to override the strategy of the map: an object uses its own strategy if it has one, otherwise
/// the one of its layer, otherwise the one of the map.
/// Colliders are spawned again when this component is inserted, modified or removed.
///
/// Since it is a regular component, it can also be set from Tiled using a custom property
/// (see the `user_properties` feature).
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// // Use triangulated colliders for the layer named "water"
/// fn on_layer_created(
///     layer_created: On<TiledEvent<LayerCreated>>,
///     layers_query: Query<&TiledName, With<TiledLayer>>,
///     mut commands: Commands,
/// ) {
///     let layer = layer_created.event().origin;
///     if layers_query.get(layer).is_ok_and(|name| name.0 == "water") {
///         commands.entity(layer).insert(TiledColliderStrategy::Triangulation);
///     }
/// }
/// ```
///
/// The description of each strategy is the one of built-in physics backends: a custom
/// [`TiledPhysicsBackend`] is free to interpret them differently.
#[derive(Component, Reflect, Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[reflect(Component, Default, Debug, PartialEq, Hash)]
pub enum TiledColliderStrategy {
    /// Aggregate all the line strings of the geometry in a single polyline collider.
    #[default]
    Polyline,
    /// Triangulate the geometry and aggregate all the triangles in a single compound collider.
    Triangulation,
    /// Create a line strip collider for each line string of the geometry: this produces several colliders.
    LineStrip,
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

/// Size, in tiles, of the chunks used to split the colliders of a [`TiledLayer::Tiles`].
///
/// The chunk size used for all the tiles layers of a map is given by [`TiledPhysicsSettings::tiles_chunk_size`].
/// Insert this [`Component`] on a [`TiledLayer::Tiles`] entity to override it for this layer only:
/// `TiledColliderChunkSize(None)` disables chunking for this layer, whatever the map setting.
/// Colliders of the layer are spawned again when this component is inserted, modified or removed.
///
/// Since it is a regular component, it can also be set from Tiled using a custom property
/// (see the `user_properties` feature).
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// // Use 16x16 tiles collider chunks for the layer named "ground"
/// fn on_layer_created(
///     layer_created: On<TiledEvent<LayerCreated>>,
///     layers_query: Query<&TiledName, With<TiledLayer>>,
///     mut commands: Commands,
/// ) {
///     let layer = layer_created.event().origin;
///     if layers_query.get(layer).is_ok_and(|name| name.0 == "ground") {
///         commands
///             .entity(layer)
///             .insert(TiledColliderChunkSize(Some(UVec2::splat(16))));
///     }
/// }
/// ```
#[derive(Component, Reflect, Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[reflect(Component, Default, Debug, PartialEq, Hash)]
pub struct TiledColliderChunkSize(pub Option<UVec2>);

/// Marker [`Component`] for the entity holding the colliders of a chunk of a [`TiledLayer::Tiles`].
///
/// Only used when the colliders of the layer are split in chunks: see [`TiledPhysicsSettings::tiles_chunk_size`].
/// A chunk entity is a child of its layer and only exists if at least one tile of the chunk has a collider.
/// It holds the first collider of the chunk: additional ones, if any, are children of the chunk entity.
///
/// The inner value is the chunk index: the chunk holds the colliders of tiles
/// whose [`TilePos`] is within `index * chunk_size .. (index + 1) * chunk_size`.
#[derive(Component, Reflect, Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[reflect(Component, Debug, PartialEq, Hash)]
#[require(Transform, Visibility)]
pub struct TiledColliderChunk(pub UVec2);

/// Raw geometry a physics collider was created from.
///
/// If several colliders are created from the same geometry (see [`TiledColliderStrategy::LineStrip`]),
/// each of them holds the whole geometry.
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
    app.register_type::<TiledColliderStrategy>();
    app.register_type::<TiledColliderChunkSize>();
    app.register_type::<TiledColliderChunk>();
    app.register_type::<TiledColliderOf>();
    app.register_type::<TiledColliders>();
    app.add_message::<TiledEvent<ColliderCreated>>()
        .register_type::<TiledEvent<ColliderCreated>>();
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
                matches!(map_asset.map_type, TilemapType::Isometric(..)),
                &map_asset.tilemap_size,
                &map_asset.grid_size,
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

/// Collider polygons of the tiles of a map, by tileset ID, [`tiled::TileId`] and [`TileFlip`].
///
/// Many tiles of a layer usually share the same tile: its polygons are only computed once,
/// then translated for each tile. A cache is only valid for a given map asset and filter.
#[derive(Default)]
pub(crate) struct TilesPolygonsCache(
    HashMap<(TilesetId, tiled::TileId, [bool; 3]), Option<CachedTilePolygons>>,
);

/// Collider polygons of a tile
struct CachedTilePolygons {
    /// Polygons, relative to the tile bottom-left corner, with tile flips applied
    polygons: Vec<geo::Polygon<f32>>,
    /// Size of the tile
    tile_size: TilemapTileSize,
    /// Offset of the tile tilemap
    offset: Vec2,
}

impl CachedTilePolygons {
    fn new(
        map_asset: &TiledMapAsset,
        tileset_id: TilesetId,
        tile_id: tiled::TileId,
        flip: &TileFlip,
        filter: &TiledFilter,
    ) -> Option<Self> {
        let tile = map_asset.tile(tileset_id, tile_id)?;
        let collision = tile.collision.as_ref()?;
        let tile_size = tile_size(&tile);
        let size = Vec2::new(tile_size.x, tile_size.y);
        let flipped = flip.x || flip.y || flip.d;
        // Compute polygons relative to the tile bottom-left corner, then apply tile flips
        let polygons = polygons_from_tile(
            map_asset,
            collision,
            filter,
            &tile_size,
            Vec2::ZERO,
            Vec2::ONE,
        )
        .into_iter()
        .map(|polygon| {
            if flipped {
                polygon.map_coords(|coord| flip_tile_coord(coord, size, flip))
            } else {
                polygon
            }
        })
        .collect::<Vec<_>>();
        if polygons.is_empty() {
            return None;
        }
        Some(Self {
            polygons,
            tile_size,
            offset: tilemap_offset(&map_asset.map, tile.tileset()),
        })
    }
}

/// Compute the collider polygons of a set of tiles, relative to their parent [`TiledLayer::Tiles`].
///
/// Tiles are given as their position, tileset ID, [`tiled::TileId`] and [`TileFlip`].
/// Polygons are computed from the tiles collision objects which match `filter`.
pub(crate) fn tiles_polygons(
    map_asset: &TiledMapAsset,
    anchor: &TilemapAnchor,
    tiles: impl Iterator<Item = (TilePos, TilesetId, tiled::TileId, TileFlip)>,
    filter: &TiledFilter,
    cache: &mut TilesPolygonsCache,
) -> Vec<geo::Polygon<f32>> {
    let mut acc = vec![];
    for (tile_pos, tileset_id, tile_id, flip) in tiles {
        // Tiles outside of map bounds are not rendered: do not give them a collider
        if !tile_pos.within_map_bounds(&map_asset.tilemap_size) {
            continue;
        }
        let Some(cached) = cache
            .0
            .entry((tileset_id, tile_id, [flip.x, flip.y, flip.d]))
            .or_insert_with(|| {
                CachedTilePolygons::new(map_asset, tileset_id, tile_id, &flip, filter)
            })
        else {
            continue;
        };
        // Move polygons to the actual tile position
        let tile_position =
            map_asset.tile_relative_position(&tile_pos, &cached.tile_size, anchor) + cached.offset;
        let bottom_left = tile_position - Vec2::new(cached.tile_size.x, cached.tile_size.y) / 2.;
        acc.extend(cached.polygons.iter().map(|polygon| {
            polygon.map_coords(|coord| geo::Coord {
                x: coord.x + bottom_left.x,
                y: coord.y + bottom_left.y,
            })
        }));
    }
    acc
}

/// Where the colliders of a source (a tiles layer or an object) are spawned.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ColliderTarget {
    /// First collider is inserted on the source entity itself, next ones on children of the source.
    Source,
    /// First collider is inserted on a new chunk entity, child of the source, next ones on children of the chunk.
    Chunk(UVec2),
}

/// Spawn colliders for the given polygons using the physics backend from `settings`.
///
/// Returns all the entities holding a collider.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_colliders<T: TiledPhysicsBackend>(
    settings: &TiledPhysicsSettings<T>,
    strategy: TiledColliderStrategy,
    target: ColliderTarget,
    commands: &mut Commands,
    polygons: Vec<geo::Polygon<f32>>,
    collider_created: TiledEvent<ColliderCreated>,
    message_writer: &mut MessageWriter<TiledEvent<ColliderCreated>>,
    disable_geometry_simplification: bool,
) -> Vec<Entity> {
    if polygons.is_empty() {
        return vec![];
    }

    // Try to simplify geometry by merging together adjacent polygons: nothing to merge for a single one
    let geometries =
        if polygons.len() > 1 && !disable_geometry_simplification && settings.simplify_geometry {
            // Boolean operations expect a consistent winding order
            let polygons = polygons
                .into_iter()
                .map(|polygon| polygon.orient(::geo::orient::Direction::Default))
                .collect::<Vec<_>>();
            vec![::geo::unary_union(&polygons)]
        } else {
            polygons
                .into_iter()
                .map(|polygon| geo::MultiPolygon::new(vec![polygon]))
                .collect()
        };

    let source = collider_created.event.source;
    let collider_of = collider_created.event.collider_of;
    // Entity holding the first collider, parent of the next ones
    let mut first = None;
    let mut entities = Vec::new();
    for geometry in geometries {
        // Actually create our colliders using provided physics backend
        for collider in settings.backend.colliders(strategy, source, &geometry) {
            let entity = match first {
                None => match target {
                    ColliderTarget::Source => collider_of.0,
                    ColliderTarget::Chunk(chunk) => commands
                        .spawn((
                            Name::new(format!("TiledColliderChunk({}, {})", chunk.x, chunk.y)),
                            TiledColliderChunk(chunk),
                            ChildOf(collider_of.0),
                        ))
                        .id(),
                },
                Some(parent) => commands
                    .spawn((
                        Name::new(format!("TiledCollider({})", entities.len())),
                        ChildOf(parent),
                    ))
                    .id(),
            };
            first.get_or_insert(entity);
            entities.push(entity);
            commands.entity(entity).insert((
                collider,
                source,
                TiledColliderPolygons(vec![geometry.clone()]),
                collider_of,
            ));

            // Patch origin entity and send collider event
            let mut event = collider_created;
            event.origin = entity;
            event.send(commands, message_writer);
        }
    }
    entities
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
