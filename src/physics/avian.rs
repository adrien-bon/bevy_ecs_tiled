//! Avian physics backend.
//!
//! Only available when the `avian` feature is enabled.

use avian2d::{
    parry::{
        math::{Isometry, Real},
        shape::SharedShape,
    },
    prelude::*,
};
use bevy::prelude::*;
use tiled::{Map, ObjectLayerData, ObjectShape};

use crate::prelude::*;

/// The actual Avian physics backend to use when instantiating the physics plugin.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default());
/// ```
#[derive(Default, Clone, Reflect)]
pub struct TiledPhysicsAvianBackend;

impl TiledPhysicsBackend for TiledPhysicsAvianBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        map: &Map,
        filter: &TiledNameFilter,
        collider: &TiledCollider,
    ) -> Vec<TiledColliderSpawnInfos> {
        match collider {
            TiledCollider::Object {
                layer_id: _,
                object_id: _,
            } => {
                let Some(object) = collider.get_object(map) else {
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
                            map.tile_width as f32,
                            map.tile_height as f32,
                            &mut composables,
                            &mut spawn_infos,
                        );
                        if !composables.is_empty() {
                            let collider: Collider = SharedShape::compound(composables).into();
                            spawn_infos.push(TiledColliderSpawnInfos {
                                name: "Avian[ComposedTile]".to_string(),
                                entity: commands.spawn(collider).id(),
                                position: Vec2::ZERO,
                                rotation: 0.,
                            });
                        }
                        Some(spawn_infos)
                    }),
                    None => get_position_and_shape(&object.shape).map(|(pos, shared_shape, _)| {
                        let collider: Collider = shared_shape.into();
                        vec![TiledColliderSpawnInfos {
                            name: format!("Avian[Object={}]", object.name),
                            entity: commands.spawn(collider).id(),
                            position: pos,
                            rotation: -object.rotation,
                        }]
                    }),
                }
                .unwrap_or_default()
            }
            TiledCollider::TilesLayer { layer_id: _ } => {
                let mut composables = vec![];
                let mut spawn_infos = vec![];
                for (tile_position, tile) in collider.get_tiles(map) {
                    if let Some(collision) = &tile.collision {
                        compose_tiles(
                            commands,
                            filter,
                            collision,
                            tile_position,
                            map.tile_width as f32,
                            map.tile_height as f32,
                            &mut composables,
                            &mut spawn_infos,
                        );
                    }
                }
                if !composables.is_empty() {
                    let collider: Collider = SharedShape::compound(composables).into();
                    spawn_infos.push(TiledColliderSpawnInfos {
                        name: "Avian[ComposedTile]".to_string(),
                        entity: commands.spawn(collider).id(),
                        position: Vec2::ZERO,
                        rotation: 0.,
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
    origin: Vec2,
    tile_width: f32,
    tile_height: f32,
    composables: &mut Vec<(Isometry<Real>, SharedShape)>,
    spawn_infos: &mut Vec<TiledColliderSpawnInfos>,
) {
    for object in object_layer_data.object_data() {
        if !filter.contains(&object.name) {
            continue;
        }
        let object_position = Vec2 {
            x: object.x - tile_width / 2.,
            y: (tile_height - object.y) - tile_height / 2.,
        };
        if let Some((mut position, shared_shape, is_composable)) =
            get_position_and_shape(&object.shape)
        {
            position += origin + object_position;
            position += Vec2 {
                x: (tile_width) / 2.,
                y: (tile_height) / 2.,
            };
            if is_composable {
                composables.push((
                    Isometry::<Real>::new(position.into(), f32::to_radians(-object.rotation)),
                    shared_shape,
                ));
            } else {
                let collider: Collider = shared_shape.into();
                spawn_infos.push(TiledColliderSpawnInfos {
                    name: "Avian[ComplexTile]".to_string(),
                    entity: commands.spawn(collider).id(),
                    position,
                    rotation: -object.rotation,
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
            let shape = SharedShape::new(EllipseColliderShape(Ellipse::new(
                width / 2.0,
                height / 2.0,
            )));
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
