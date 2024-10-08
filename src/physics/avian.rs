use avian2d::{math::Vector, prelude::*};
use bevy::prelude::*;
use tiled::{Map, ObjectData};

use crate::prelude::*;

#[derive(Default)]
pub struct TiledPhysicsAvianBackend;

impl super::TiledPhysicsBackend for TiledPhysicsAvianBackend {
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
                let shape = Collider::rectangle(*width, *height);
                let pos = Vector::new(width / 2., -height / 2.);
                (pos, shape)
            }
            tiled::ObjectShape::Ellipse { width, height } => {
                let shape = Collider::ellipse(width / 2., height / 2.);
                let pos = Vector::new(width / 2., -height / 2.);
                (pos, shape)
            }
            tiled::ObjectShape::Polyline { points } => {
                let shape = Collider::polyline(
                    points.iter().map(|(x, y)| Vector::new(*x, -*y)).collect(),
                    None,
                );
                (Vector::ZERO, shape)
            }
            tiled::ObjectShape::Polygon { points } => {
                let shape = match Collider::convex_hull(
                    points
                        .iter()
                        .map(|(x, y)| Vector::new(*x, -*y))
                        .collect::<Vec<Vector>>(),
                ) {
                    Some(x) => x,
                    None => {
                        return None;
                    }
                };
    
                (Vector::ZERO, shape)
            }
            _ => {
                return None;
            }
        };
        Some((pos, commands.spawn(collider).id()))
    }
}
