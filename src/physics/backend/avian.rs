//! Avian physics backend for bevy_ecs_tiled.
//!
//! This module provides an implementation of the [`TiledPhysicsBackend`] trait using the Avian 2D physics engine.
//! This backend is only available when the `avian` feature is enabled.
//!
//! # Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_ecs_tiled::prelude::*;
//!
//! App::new()
//!     .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default());
//! ```

use crate::{prelude::*, tiled::helpers::global_transform_from_isometry_2d};
use avian2d::{
    collision::collider::EllipseColliderShape,
    parry::{
        math::{Isometry, Real},
        shape::SharedShape,
    },
    prelude::*,
};
use bevy::prelude::*;
use tiled::{ObjectData, ObjectLayerData};

/// The [`TiledPhysicsBackend`] to use for Avian 2D integration.
#[derive(Default, Reflect, Copy, Clone, Debug)]
#[reflect(Default, Debug)]
pub struct TiledPhysicsAvianBackend;

impl TiledPhysicsBackend for TiledPhysicsAvianBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        assets: &Res<Assets<TiledMapAsset>>,
        anchor: &TilemapAnchor,
        filter: &TiledNameFilter,
        source: &TiledEvent<ColliderCreated>,
    ) -> Vec<TiledPhysicsBackendOutput> {
        let Some(map_asset) = source.get_map_asset(assets) else {
            return vec![];
        };

        let mut composables = vec![];
        let mut spawn_infos = match source.event.0 {
            TiledCollider::Object => {
                let Some(object) = source.get_object(assets) else {
                    return vec![];
                };

                match object.get_tile() {
                    // If the object has a tile, we need to handle its collision data
                    Some(object_tile) => object_tile.get_tile().map(|tile| {
                        let Some(object_layer_data) = &tile.collision else {
                            return vec![];
                        };
                        let grid_size = TilemapGridSize::new(
                            tile.tileset().tile_width as f32,
                            tile.tileset().tile_height as f32,
                        );
                        colliders_from_tile(
                            commands,
                            object_layer_data,
                            grid_size,
                            filter,
                            Vec2::ZERO,
                            &mut composables,
                        )
                    }),
                    // If the object does not have a tile, we can create a collider directly from itself
                    None => collider_from_object(&object, &GlobalTransform::default()).map(
                        |(isometry_2d, shared_shape, _)| {
                            let collider: Collider = shared_shape.into();
                            vec![TiledPhysicsBackendOutput {
                                name: format!("Avian[Object={}]", object.name),
                                entity: commands.spawn(collider).id(),
                                transform: global_transform_from_isometry_2d(&isometry_2d).into(),
                            }]
                        },
                    ),
                }
                .unwrap_or_default()
            }
            TiledCollider::TilesLayer => {
                let grid_size = grid_size_from_map(&map_asset.map);
                let mut acc = vec![];
                // Iterate over all tiles in the layer and create colliders for each
                for (tile_position, tile) in source.get_tiles(assets, anchor) {
                    if let Some(collision) = &tile.collision {
                        acc.extend(colliders_from_tile(
                            commands,
                            collision,
                            grid_size,
                            filter,
                            Vec2::new(
                                tile_position.x - grid_size.x / 2.,
                                tile_position.y - grid_size.y / 2.,
                            ),
                            &mut composables,
                        ));
                    }
                }
                acc
            }
        };

        // If we have composable shapes, we need to create a compound collider
        if !composables.is_empty() {
            let collider: Collider = SharedShape::compound(composables).into();
            spawn_infos.push(TiledPhysicsBackendOutput {
                name: "Avian[ComposedTile]".to_string(),
                entity: commands.spawn(collider).id(),
                transform: Transform::default(),
            });
        }
        spawn_infos
    }
}

fn collider_from_object(
    object_data: &ObjectData,
    transform: &GlobalTransform,
) -> Option<(Isometry2d, SharedShape, bool)> {
    let tiled_object = TiledObject::from_object_data(object_data);
    match &tiled_object {
        TiledObject::Point | TiledObject::Text | TiledObject::Tile { .. } => None,
        TiledObject::Rectangle { width, height } => Some((
            tiled_object.isometry_2d(transform),
            SharedShape::cuboid(width / 2., height / 2.),
            true,
        )),
        TiledObject::Ellipse { width, height } => Some((
            tiled_object.isometry_2d(transform),
            SharedShape::new(EllipseColliderShape(Ellipse::new(width / 2., height / 2.))),
            true,
        )),
        TiledObject::Polygon { vertices: _ } => {
            let vertices = tiled_object.vertices(transform);
            if vertices.len() < 3 {
                return None;
            }
            let indices = (0..vertices.len() as u32 - 1)
                .map(|i| [i, i + 1])
                .chain([[vertices.len() as u32 - 1, 0]])
                .collect();
            let vertices = vertices.iter().map(|v| (*v).into()).collect();
            Some((
                tiled_object.isometry_2d(transform),
                SharedShape::polyline(vertices, Some(indices)),
                false,
            ))
        }
        TiledObject::Polyline { vertices: _ } => {
            let vertices = tiled_object.vertices(transform);
            let vertices = vertices.iter().map(|v| (*v).into()).collect();
            Some((
                tiled_object.isometry_2d(transform),
                SharedShape::polyline(vertices, None),
                false,
            ))
        }
    }
}

fn colliders_from_tile(
    commands: &mut Commands,
    object_layer_data: &ObjectLayerData,
    grid_size: TilemapGridSize,
    filter: &TiledNameFilter,
    offset: Vec2,
    composables: &mut Vec<(Isometry<Real>, SharedShape)>,
) -> Vec<TiledPhysicsBackendOutput> {
    let mut output = Vec::new();
    for object in object_layer_data.object_data() {
        if !filter.contains(&object.name) {
            continue;
        }

        let transform = global_transform_from_isometry_2d(&Isometry2d {
            rotation: Rot2::radians(f32::to_radians(-object.rotation)),
            translation: Vec2::new(offset.x + object.x, offset.y + grid_size.y - object.y),
        });

        match collider_from_object(object, &transform) {
            // We have a collider that can be composed: add it to the composables list
            Some((isometry_2d, shared_shape, true)) => {
                composables.push((
                    Isometry::<Real>::new(
                        Vec2::new(isometry_2d.translation.x, isometry_2d.translation.y).into(),
                        isometry_2d.rotation.as_radians(),
                    ),
                    shared_shape,
                ));
            }
            // We have a collider that cannot be composed: spawn it directly and add it to the output
            Some((isometry_2d, shared_shape, false)) => {
                output.push(TiledPhysicsBackendOutput {
                    name: format!("Avian[Object={}]", object.name),
                    entity: commands.spawn(Collider::from(shared_shape)).id(),
                    transform: global_transform_from_isometry_2d(&isometry_2d)
                        .reparented_to(&transform),
                });
            }
            None => {}
        }
    }
    output
}
