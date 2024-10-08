use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use tiled::{Map, ObjectData};

use crate::prelude::*;

#[derive(Default)]
pub struct TiledPhysicsRapierBackend;

impl super::TiledPhysicsBackend for TiledPhysicsRapierBackend {
    fn spawn_collider(
        &self,
        commands: &mut Commands,
        _map: &Map,
        _collider_source: &TiledColliderSource,
        object_data: &ObjectData,
    ) -> Option<(Vec2, Entity)> {
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
        Some((pos, commands.spawn(collider).id()))
    }
}
