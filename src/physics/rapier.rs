use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use tiled::Map;

use crate::prelude::*;

#[derive(Default)]
pub struct TiledPhysicsRapierBackend;

impl super::TiledPhysicsBackend for TiledPhysicsRapierBackend {
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

        let object_data = (match collider_source {
            TiledColliderSource::Tile {
                layer_id: _,
                x: _,
                y: _,
                object_id,
            } => tile
                .as_ref()
                .and_then(|tile| tile.collision.as_ref())
                .map(|collision| collision.object_data())
                .and_then(|objects| objects.get(*object_id)),
            TiledColliderSource::Object {
                layer_id: _,
                object_id: _,
            } => object.as_deref(),
        })?;

        let (pos, collider) = match &object_data.shape {
            tiled::ObjectShape::Rect { width, height } => {
                // The origin is the top-left corner of the rectangle when not rotated.
                let shape = Collider::cuboid(width / 2., height / 2.);
                let pos = Vect::new(width / 2., -height / 2.);
                (pos, shape)
            }
            tiled::ObjectShape::Ellipse { width, height } => {
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
                (pos, shape)
            }
            tiled::ObjectShape::Polyline { points } => {
                let shape = Collider::polyline(
                    points.iter().map(|(x, y)| Vect::new(*x, -*y)).collect(),
                    None,
                );
                (Vect::ZERO, shape)
            }
            tiled::ObjectShape::Polygon { points } => {
                let shape = match Collider::convex_hull(
                    &points
                        .iter()
                        .map(|(x, y)| Vect::new(*x, -*y))
                        .collect::<Vec<_>>(),
                ) {
                    Some(x) => x,
                    None => {
                        return None;
                    }
                };

                (Vect::ZERO, shape)
            }
            _ => {
                return None;
            }
        };
        Some(TiledColliderSpawnInfos {
            name: format!("Rapier[{}]", object_data.name),
            entity: commands.spawn(collider).id(),
            position: pos,
            rotation: -object_data.rotation,
        })
    }
}
