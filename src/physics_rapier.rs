use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use tiled::ObjectLayer;

/// Loads shapes from an object layer as physics colliders.
///
/// TODO: This is not ideal because object layers could be used for many
///       things, not just collisions. We shouldn't assume a shape is intended
///       for a collider.
pub fn load_physics_layer(
    commands: &mut Commands,
    layer_entity: Entity,
    object_layer: ObjectLayer,
    map_size: TilemapSize,
    grid_size: TilemapGridSize,
    offset_x: f32,
    offset_y: f32,
) {
    for object_data in object_layer.objects() {
        let pos = Vect::new(
            object_data.x,
            (map_size.y as f32 * grid_size.y) - object_data.y,
        );
        let rot = object_data.rotation;
        let shape = match &object_data.shape {
            tiled::ObjectShape::Rect { width, height } => {
                let shape = Collider::cuboid(width / 2., height / 2.);
                Some((
                    pos + Vec2::new((-grid_size.x + width) / 2., (-grid_size.y - height) / 2.),
                    rot,
                    shape,
                ))
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

                Some((
                    pos + Vec2::new((-grid_size.x + width) / 2., (-grid_size.y - height) / 2.),
                    rot,
                    shape,
                ))
            }
            tiled::ObjectShape::Polyline { points } => {
                let shape = Collider::polyline(
                    points.iter().map(|(x, y)| Vect::new(*x, -*y)).collect(),
                    None,
                );
                Some((
                    pos + Vec2::new(-grid_size.x / 2., -grid_size.y / 2.),
                    rot,
                    shape,
                ))
            }
            tiled::ObjectShape::Polygon { points } => Collider::convex_hull(
                &points
                    .iter()
                    .map(|(x, y)| Vect::new(*x, -*y))
                    .collect::<Vec<_>>(),
            )
            .map(|shape| {
                (
                    pos + Vec2::new(-grid_size.x / 2., -grid_size.y / 2.),
                    rot,
                    shape,
                )
            }),
            _ => None,
        };

        let world_pos = Vec2 { x: 0., y: 0. };
        let transform = Transform::from_xyz(world_pos.x + offset_x, world_pos.y - offset_y, 0.);
        if let Some((pos, rot, collider)) = shape {
            let transform = transform
                * Transform {
                    translation: Vec3::new(pos.x, pos.y, 0.),
                    rotation: Quat::from_rotation_x(rot),
                    ..default()
                };
            commands
                .spawn((collider, TransformBundle::from_transform(transform)))
                .set_parent(layer_entity);
        }
    }
}
