use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use tiled::{ObjectData, ObjectLayer, ObjectLayerData};

use crate::{names::ObjectNameFilter, prelude::ColliderCallback};

/// Load shapes from an object layer as physics colliders.
///
/// By default `bevy_ecs_tiled` will only process object layers
/// named in `collision_layer_names` in `TiledMapSettings`,
/// and tileset collision shapes named in `collision_object_names`.
///
/// Collision layer names are case-insensitive and leading/trailing
/// whitespace is stripped out.
pub(crate) fn load_object_layer(
    commands: &mut Commands,
    collision_object_names: &ObjectNameFilter,
    layer_entity: Entity,
    object_layer: &ObjectLayer,
    map_size: TilemapSize,
    grid_size: TilemapGridSize,
    offset: Vec2,
    collider_callback: &ColliderCallback,
) {
    insert_colliders_from_shapes(
        commands,
        collision_object_names,
        layer_entity,
        grid_size,
        object_layer.objects().map(|x| (*x).clone()),
        map_size.y as f32 * grid_size.y,
        offset,
        collider_callback,
    );
}

pub(crate) fn insert_tile_colliders(
    commands: &mut Commands,
    collision_object_names: &ObjectNameFilter,
    tile_entity: Entity,
    grid_size: TilemapGridSize,
    collision: &ObjectLayerData,
    collider_callback: &ColliderCallback,
) {
    // commands,
    insert_colliders_from_shapes(
        commands,
        collision_object_names,
        tile_entity,
        grid_size,
        collision.object_data().iter().cloned(),
        grid_size.y,
        Vec2::ZERO,
        collider_callback,
    );
}

/// Insert shapes as physics colliders.
pub(crate) fn insert_colliders_from_shapes(
    commands: &mut Commands,
    collision_object_names: &ObjectNameFilter,
    parent_entity: Entity,
    grid_size: TilemapGridSize,
    objects: impl Iterator<Item = ObjectData>,
    max_y: f32,
    offset: Vec2,
    collider_callback: &ColliderCallback,
) {
    for object_data in objects {
        if !collision_object_names.contains(&object_data.name.trim().to_lowercase()) {
            continue;
        }

        let origin_pos = Vect::new(object_data.x, max_y - object_data.y);

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
                        continue;
                    }
                };

                (Vect::ZERO, shape)
            }
            _ => {
                continue;
            }
        };

        let center_pos = Vect::new(-grid_size.x / 2., -grid_size.y / 2.);
        let transform = Transform {
            translation: Vec3::new(
                offset.x + center_pos.x + origin_pos.x,
                -offset.y + center_pos.y + origin_pos.y,
                0.,
            ),
            rotation: Quat::from_rotation_z(f32::to_radians(-rot)),
            ..default()
        } * Transform::from_translation(Vec3::new(pos.x, pos.y, 0.));

        let mut entity_commands = commands.spawn((collider, TransformBundle::from_transform(transform)));
        entity_commands.set_parent(parent_entity);
        collider_callback(&mut entity_commands);
    }
}
