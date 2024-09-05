use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use tiled::ObjectData;

use crate::prelude::*;

/// Insert shapes as physics colliders.
pub(crate) fn insert_rapier_colliders_from_shapes<'a>(
    commands: &'a mut Commands,
    parent_entity: Entity,
    _map_type: &TilemapType,
    grid_size: Option<&TilemapGridSize>,
    object_data: &ObjectData,
) -> Option<EntityCommands<'a>> {
    let rot = object_data.rotation;
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

    let mut translation = Vec3::default();
    // If we have a grid_size, it means we are adding colliders for a tile:
    // we need to take into account object position, which are relative to the tile
    // If we don't have a grid_size, it means we are adding colliders for a standalone object
    // we need to ignore object position, since our parent should already have the correct position
    if let Some(grid_size) = grid_size {
        translation = Vec3::new(
            object_data.x - grid_size.x / 2.,
            (grid_size.y - object_data.y) - grid_size.y / 2.,
            0.,
        );
    }

    let transform = Transform {
        translation,
        rotation: Quat::from_rotation_z(f32::to_radians(-rot)),
        ..default()
    } * Transform::from_translation(Vec3::new(pos.x, pos.y, 0.));

    let mut entity_commands = commands.spawn(collider);
    entity_commands
        .insert(TransformBundle::from_transform(transform))
        .insert(Name::new(format!("Collider({})", object_data.name)))
        .set_parent(parent_entity);
    Some(entity_commands)
}
