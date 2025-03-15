//! Rapier physics backend.
//!
//! Only available when the `rapier` feature is enabled.

use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapGridSize;
use bevy_rapier2d::{
    prelude::*,
    rapier::prelude::{Isometry, Real, SharedShape},
};
use tiled::{ObjectLayerData, ObjectShape};

use crate::prelude::*;

/// The actual Rapier physics backend to use when instantiating the physics plugin.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledPhysicsPlugin::<TiledPhysicsRapierBackend>::default());
/// ```
#[derive(Default, Reflect, Copy, Clone, Debug)]
#[reflect(Default, Debug)]
pub struct TiledPhysicsRapierBackend;

impl TiledPhysicsBackend for TiledPhysicsRapierBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        tiled_map: &TiledMap,
        filter: &TiledNameFilter,
        collider: &TiledCollider,
    ) -> Vec<TiledColliderSpawnInfos> {
        match collider {
            TiledCollider::Object {
                layer_id: _,
                object_id: _,
            } => {
                let Some(object) = collider.get_object(tiled_map) else {
                    return vec![];
                };

                match object.get_tile() {
                    Some(object_tile) => object_tile.get_tile().and_then(|tile| {
                        let Some(object_layer_data) = &tile.collision else {
                            return None;
                        };
                        let mut composables = vec![];
                        let mut spawn_infos = vec![];
                        compose_tiles(
                            commands,
                            filter,
                            object_layer_data,
                            Vec2::ZERO,
                            get_grid_size(&tiled_map.map),
                            &mut composables,
                            &mut spawn_infos,
                        );
                        if !composables.is_empty() {
                            let collider: Collider = SharedShape::compound(composables).into();
                            spawn_infos.push(TiledColliderSpawnInfos {
                                name: "Rapier[ComposedTile]".to_string(),
                                entity: commands.spawn(collider).id(),
                                transform: Transform::default(),
                            });
                        }
                        Some(spawn_infos)
                    }),
                    None => get_position_and_shape(&object.shape).map(|(pos, shared_shape, _)| {
                        let collider: Collider = shared_shape.into();
                        let iso = Isometry3d::from_rotation(Quat::from_rotation_z(
                            f32::to_radians(-object.rotation),
                        )) * Isometry3d::from_xyz(pos.x, pos.y, 0.);
                        vec![TiledColliderSpawnInfos {
                            name: format!("Rapier[Object={}]", object.name),
                            entity: commands.spawn(collider).id(),
                            transform: Transform::from_isometry(iso),
                        }]
                    }),
                }
                .unwrap_or_default()
            }
            TiledCollider::TilesLayer { layer_id: _ } => {
                let mut composables = vec![];
                let mut spawn_infos = vec![];
                for (tile_position, tile) in collider.get_tiles(tiled_map) {
                    if let Some(collision) = &tile.collision {
                        compose_tiles(
                            commands,
                            filter,
                            collision,
                            tile_position,
                            get_grid_size(&tiled_map.map),
                            &mut composables,
                            &mut spawn_infos,
                        );
                    }
                }
                if !composables.is_empty() {
                    let collider: Collider = SharedShape::compound(composables).into();
                    spawn_infos.push(TiledColliderSpawnInfos {
                        name: "Rapier[ComposedTile]".to_string(),
                        entity: commands.spawn(collider).id(),
                        transform: Transform::default(),
                    });
                }
                spawn_infos
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compose_tiles(
    commands: &mut Commands,
    filter: &TiledNameFilter,
    object_layer_data: &ObjectLayerData,
    tile_offset: Vec2,
    grid_size: TilemapGridSize,
    composables: &mut Vec<(Isometry<Real>, SharedShape)>,
    spawn_infos: &mut Vec<TiledColliderSpawnInfos>,
) {
    for object in object_layer_data.object_data() {
        if !filter.contains(&object.name) {
            continue;
        }
        let object_position = Vec2 {
            x: object.x - grid_size.x / 2.,
            y: (grid_size.y - object.y) - grid_size.y / 2.,
        };
        if let Some((shape_offset, shared_shape, is_composable)) =
            get_position_and_shape(&object.shape)
        {
            let mut position = tile_offset + object_position;
            position += Vec2 {
                x: grid_size.x / 2.,
                y: grid_size.y / 2.,
            };
            if is_composable {
                composables.push((
                    Isometry::<Real>::new(position.into(), f32::to_radians(-object.rotation))
                        * Isometry::<Real>::new(shape_offset.into(), 0.),
                    shared_shape,
                ));
            } else {
                let collider: Collider = shared_shape.into();
                let iso = Isometry3d::from_xyz(position.x, position.y, 0.)
                    * Isometry3d::from_rotation(Quat::from_rotation_z(f32::to_radians(
                        -object.rotation,
                    )));
                spawn_infos.push(TiledColliderSpawnInfos {
                    name: "Rapier[ComplexTile]".to_string(),
                    entity: commands.spawn(collider).id(),
                    transform: Transform::from_isometry(iso),
                });
            }
        }
    }
}

fn get_position_and_shape(shape: &ObjectShape) -> Option<(Vec2, SharedShape, bool)> {
    match shape {
        ObjectShape::Rect { width, height } => {
            let shape = SharedShape::cuboid(width / 2., height / 2.);
            let pos = Vec2::new(width / 2., -height / 2.);
            Some((pos, shape, true))
        }
        ObjectShape::Ellipse { width, height } => {
            let shape = if width > height {
                SharedShape::capsule(
                    Vec2::new((-width + height) / 2., 0.).into(),
                    Vec2::new((width - height) / 2., 0.).into(),
                    height / 2.,
                )
            } else {
                SharedShape::capsule(
                    Vec2::new(0., (-height + width) / 2.).into(),
                    Vec2::new(0., (height - width) / 2.).into(),
                    width / 2.,
                )
            };
            let pos = Vec2::new(width / 2., -height / 2.);
            Some((pos, shape, true))
        }
        ObjectShape::Polyline { points } => {
            let vertices = points
                .iter()
                .map(|(x, y)| Vec2::new(*x, -*y))
                .map(|v| v.into())
                .collect();
            let shape = SharedShape::polyline(vertices, None);
            Some((Vec2::ZERO, shape, false))
        }
        ObjectShape::Polygon { points } => {
            if points.len() < 3 {
                return None;
            }

            let vertices = points
                .iter()
                .map(|(x, y)| Vec2::new(*x, -*y))
                .map(|v| v.into())
                .collect();
            let indices = (0..points.len() as u32 - 1)
                .map(|i| [i, i + 1])
                .chain([[points.len() as u32 - 1, 0]])
                .collect();
            let shape = SharedShape::polyline(vertices, Some(indices));
            Some((Vec2::ZERO, shape, false))
        }
        _ => None,
    }
}
