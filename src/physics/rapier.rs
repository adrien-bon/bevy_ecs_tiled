//! Rapier physics backend.
//!
//! Only available when the `rapier` feature is enabled.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
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
#[derive(Default, Clone)]
pub struct TiledPhysicsRapierBackend;

impl TiledPhysicsBackend for TiledPhysicsRapierBackend {
    fn spawn_collider(
        &self,
        commands: &mut Commands,
        map: &Map,
        collider_source: &TiledColliderSource,
    ) -> Option<TiledColliderSpawnInfos> {
        // TODO: use this function once I figure out how to prevent cloning ObjectData
        // let object_data = collider_source.object_data(map)?;

        let tile = collider_source.tile(map);
        let object = collider_source.object(map);

        let object_data = (match collider_source.ty {
            TiledColliderSourceType::Tile {
                layer_id: _,
                x: _,
                y: _,
                object_id,
            } => tile
                .as_ref()
                .and_then(|tile| tile.collision.as_ref())
                .map(|collision| collision.object_data())
                .and_then(|objects| objects.get(object_id)),
            TiledColliderSourceType::Object {
                layer_id: _,
                object_id: _,
            } => object.as_deref(),
        })?;

        let (pos, collider) = get_position_and_collider(&object_data.shape)?;

        Some(TiledColliderSpawnInfos {
            name: format!("Rapier[{}]", object_data.name),
            entity: commands.spawn(collider).id(),
            position: pos,
            rotation: -object_data.rotation,
        })
    }
}

fn get_position_and_collider(shape: &ObjectShape) -> Option<(Vect, Collider)> {
    match shape {
        ObjectShape::Rect { width, height } => {
            // The origin is the top-left corner of the rectangle when not rotated.
            let shape = Collider::cuboid(width / 2., height / 2.);
            let pos = Vect::new(width / 2., -height / 2.);
            Some((pos, shape))
        }
        ObjectShape::Ellipse { width, height } => {
            let shape = if width > height {
                Collider::capsule(
                    Vec2::new((-width + height) / 2., 0.),
                    Vec2::new((width - height) / 2., 0.),
                    height / 2.,
                )
            } else {
                Collider::capsule(
                    Vec2::new(0., (-height + width) / 2.),
                    Vec2::new(0., (height - width) / 2.),
                    width / 2.,
                )
            };

            let pos = Vect::new(width / 2., -height / 2.);
            Some((pos, shape))
        }
        ObjectShape::Polyline { points } => {
            let shape = Collider::polyline(
                points.iter().map(|(x, y)| Vect::new(*x, -*y)).collect(),
                None,
            );
            Some((Vect::ZERO, shape))
        }
        ObjectShape::Polygon { points } => {
            if points.len() < 3 {
                return None;
            }

            let points = points
                .iter()
                .map(|(x, y)| Vect::new(*x, -*y))
                .collect::<Vec<_>>();

            let indices = (0..points.len() as u32 - 1)
                .map(|i| [i, i + 1])
                .chain([[points.len() as u32 - 1, 0]])
                .collect();

            let shape = Collider::polyline(points, Some(indices));

            Some((Vect::ZERO, shape))
        }
        _ => None,
    }
}
