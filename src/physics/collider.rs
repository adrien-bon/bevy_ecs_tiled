//! Collider management for Tiled maps and worlds.
//!
//! This module defines marker components and events for colliders generated from Tiled maps and objects.
//! It provides types to distinguish between colliders created from tile layers and object layers,
//! as well as utilities for extracting tile data relevant to collider generation.

use std::collections::VecDeque;

use crate::prelude::*;
use crate::tiled::helpers::iso_projection;
use bevy::prelude::*;
use geo::BooleanOps;
use tiled::{ObjectLayerData, ObjectShape};

/// Marker component for colliders origin
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

/// Collider raw geometry
#[derive(Component, PartialEq, Clone, Debug, Deref)]
#[require(Transform)]
pub struct TiledColliderPolygons(pub MultiPolygon<f32>);

/// Event emitted when a collider is created from a Tiled map or world.
///
/// You can determine collider origin using the inner [`TiledColliderOrigin`].
/// See also [`TiledEvent`]
#[derive(Clone, Copy, PartialEq, Debug, Reflect, Deref)]
#[reflect(Clone, PartialEq)]
pub struct ColliderCreated(pub TiledColliderOrigin);

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledColliderOrigin>();
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
                        let grid_size = grid_size_from_map(&map_asset.map);
                        let tile_size = tile_size_from_grid_size(&grid_size);
                        let map_type = tilemap_type_from_map(&map_asset.map);
                        let tile_coords = tile_pos.center_in_world(
                            &map_asset.tilemap_size,
                            &grid_size,
                            &tile_size,
                            &map_type,
                            anchor,
                        );
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

                        // For isometric maps, we need to project the object's vertices onto the isometric plane
                        if matches!(
                            tilemap_type_from_map(&map_asset.map),
                            TilemapType::Isometric(..)
                        ) {
                            create_isometric_object_polygon(
                                &object,
                                global_transform,
                                map_asset,
                                anchor,
                            )
                            .map(|p| vec![p])
                        } else {
                            TiledObject::from_object_data(&object)
                                .polygon(global_transform)
                                .map(|p| vec![p])
                        }
                    }
                    // If the object has a tile, we need to handle its collision data
                    Some(object_tile) => object_tile.get_tile().map(|tile| {
                        let Some(object_layer_data) = &tile.collision else {
                            return vec![];
                        };
                        let ObjectShape::Rect { width, height } = object.shape else {
                            return vec![];
                        };

                        let unscaled_tile_size = match &tile.image {
                            Some(image) => {
                                // tile is in image collection
                                Vec2::new(image.width as f32, image.height as f32)
                            }
                            None => Vec2::new(
                                tile.tileset().tile_width as f32,
                                tile.tileset().tile_height as f32,
                            ),
                        };

                        let mut scale = Vec2::new(width, height) / unscaled_tile_size;
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
                            TilemapGridSize::new(width, height),
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
            let grid_size = grid_size_from_map(&map_asset.map);
            let mut acc = vec![];
            // Iterate over all tiles in the layer and create colliders for each
            for (tile_position, tile) in source.get_tiles(assets, anchor) {
                if let Some(collision) = &tile.collision {
                    acc.extend(polygons_from_tile(
                        collision,
                        filter,
                        grid_size,
                        Vec2::new(
                            tile_position.x - grid_size.x / 2.,
                            tile_position.y - grid_size.y / 2.,
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
    grid_size: TilemapGridSize,
    offset: Vec2,
    scale: Vec2,
) -> Vec<GeoPolygon<f32>> {
    let mut polygons = vec![];
    for object in object_layer_data.object_data() {
        if !filter.matches(&object.name) {
            continue;
        }

        let pos = offset + Vec2::new(object.x * scale.x, grid_size.y - object.y * scale.y);
        let transform = GlobalTransform::from_isometry(Isometry3d {
            rotation: Quat::from_rotation_z(f32::to_radians(-object.rotation)),
            translation: pos.extend(0.).into(),
        }) * Transform::from_scale(scale.extend(1.));

        if let Some(p) = TiledObject::from_object_data(object).polygon(&transform) {
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

/// Creates a polygon for an isometric object by projecting all vertices onto the isometric plane.
///
/// This is necessary because Tiled objects on isometric maps need their shapes projected,
/// not just their positions transformed.
fn create_isometric_object_polygon(
    object: &Object,
    global_transform: &GlobalTransform,
    map_asset: &TiledMapAsset,
    _anchor: &TilemapAnchor,
) -> Option<GeoPolygon<f32>> {
    let grid_size = grid_size_from_map(&map_asset.map);
    let tilemap_size = map_asset.tilemap_size;

    // Get the object's vertices in Tiled coordinate space (relative to object origin)
    let tiled_object = TiledObject::from_object_data(object);

    // Use the object's actual world position from its transform
    // This accounts for any adjustments made during object spawning (like the tile object width offset)
    let object_world_pos = Vec2::new(
        global_transform.translation().x,
        global_transform.translation().y,
    );

    // Create vertices by projecting the shape relative to origin, then offset to correct position
    let final_vertices: Vec<Coord<f32>> = match tiled_object {
        TiledObject::Rectangle { width, height } => {
            // Convert rectangle to polygon vertices relative to object origin
            let corners = vec![
                Vec2::new(0.0, 0.0),      // Top-left relative to object
                Vec2::new(width, 0.0),    // Top-right
                Vec2::new(width, height), // Bottom-right
                Vec2::new(0.0, height),   // Bottom-left
            ];

            corners
                .into_iter()
                .map(|corner| {
                    // Project the relative offset through isometric transformation
                    let projected_offset =
                        iso_projection(corner + map_asset.tiled_offset, &tilemap_size, &grid_size);
                    let origin_projected =
                        iso_projection(map_asset.tiled_offset, &tilemap_size, &grid_size);
                    let relative_projected = projected_offset - origin_projected;

                    Coord {
                        x: object_world_pos.x + relative_projected.x,
                        y: object_world_pos.y - relative_projected.y, // Flip Y to match Bevy coordinate system
                    }
                })
                .collect()
        }
        TiledObject::Ellipse { width, height } => {
            // For ellipses, create multiple points to approximate the shape
            const NUM_POINTS: u32 = 20;
            (0..NUM_POINTS)
                .map(|i| {
                    let theta = 2.0 * std::f32::consts::PI * (i as f32) / (NUM_POINTS as f32);
                    let local_x = width / 2.0 * theta.cos() + width / 2.0;
                    let local_y = height / 2.0 * theta.sin() + height / 2.0;
                    let corner = Vec2::new(local_x, local_y);

                    // Project the relative offset through isometric transformation
                    let projected_offset =
                        iso_projection(corner + map_asset.tiled_offset, &tilemap_size, &grid_size);
                    let origin_projected =
                        iso_projection(map_asset.tiled_offset, &tilemap_size, &grid_size);
                    let relative_projected = projected_offset - origin_projected;

                    Coord {
                        x: object_world_pos.x + relative_projected.x,
                        y: object_world_pos.y - relative_projected.y,
                    }
                })
                .collect()
        }
        TiledObject::Polygon { vertices } => {
            vertices
                .into_iter()
                .map(|vertex| {
                    let corner = Vec2::new(vertex.x, -vertex.y); // Note: -vertex.y due to Tiled Y-flip

                    // Project the relative offset through isometric transformation
                    let projected_offset =
                        iso_projection(corner + map_asset.tiled_offset, &tilemap_size, &grid_size);
                    let origin_projected =
                        iso_projection(map_asset.tiled_offset, &tilemap_size, &grid_size);
                    let relative_projected = projected_offset - origin_projected;

                    Coord {
                        x: object_world_pos.x + relative_projected.x,
                        y: object_world_pos.y - relative_projected.y,
                    }
                })
                .collect()
        }
        TiledObject::Point => {
            // Points don't create polygons
            return None;
        }
        TiledObject::Polyline { .. } => {
            // Polylines aren't closed shapes
            return None;
        }
        TiledObject::Tile { .. } | TiledObject::Text => {
            // These should be handled elsewhere
            return None;
        }
    };

    if final_vertices.len() >= 3 {
        Some(GeoPolygon::new(LineString::new(final_vertices), vec![]))
    } else {
        None
    }
}
