//! Rapier physics backend.
//!
//! Only available when the `rapier` feature is enabled.

use bevy::prelude::*;
use bevy_rapier2d::{
    prelude::*,
    rapier::prelude::{Isometry, Real, SharedShape},
};
use tiled::{Map, ObjectShape};

use crate::prelude::*;

/// The actual Rapier physics backend to use when instantiating the physics plugin.
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
pub struct TiledPhysicsRapierBackend;

impl TiledPhysicsBackend for TiledPhysicsRapierBackend {
    fn spawn_collider(
        &self,
        commands: &mut Commands,
        map: &Map,
        collider_source: &TiledColliderSource,
    ) -> Vec<TiledColliderSpawnInfos> {
        match collider_source.ty {
            TiledColliderSourceType::Object {
                layer_id: _,
                object_id: _,
            } => {
                let Some(object) = collider_source.get_object(map) else {
                    return vec![];
                };
                let Some((pos, shared_shape, _)) = get_position_and_shape(&object.shape) else {
                    return vec![];
                };
                let collider: Collider = shared_shape.into();
                vec![TiledColliderSpawnInfos {
                    name: format!("Rapier[Object={}]", object.name),
                    entity: commands.spawn(collider).id(),
                    position: pos,
                    rotation: -object.rotation,
                }]
            }
            TiledColliderSourceType::TilesLayer { layer_id: _ } => {
                let mut composables = vec![];
                let mut spawn_infos = vec![];
                for (tile_position, tile) in collider_source.get_tiles(map) {
                    if let Some(collision) = &tile.collision {
                        for object in collision.object_data() {
                            let object_position = Vec2 {
                                x: object.x - map.tile_width as f32 / 2.,
                                y: (map.tile_height as f32 - object.y)
                                    - map.tile_height as f32 / 2.,
                            };
                            if let Some((mut position, shared_shape, is_composable)) =
                                get_position_and_shape(&object.shape)
                            {
                                position += tile_position + object_position;
                                position += Vec2 {
                                    x: (map.tile_width as f32) / 2.,
                                    y: (map.tile_height as f32) / 2.,
                                };
                                if is_composable {
                                    composables.push((
                                        Isometry::<Real>::new(
                                            position.into(),
                                            f32::to_radians(-object.rotation),
                                        ),
                                        shared_shape,
                                    ));
                                } else {
                                    let collider: Collider = shared_shape.into();
                                    spawn_infos.push(TiledColliderSpawnInfos {
                                        name: "Rapier[ComplexTile]".to_string(),
                                        entity: commands.spawn(collider).id(),
                                        position,
                                        rotation: -object.rotation,
                                    });
                                }
                            }
                        }
                    }
                }
                if !composables.is_empty() {
                    let collider: Collider = SharedShape::compound(composables).into();
                    spawn_infos.push(TiledColliderSpawnInfos {
                        name: "Rapier[ComposedTile]".to_string(),
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
