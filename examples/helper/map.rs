use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

const ROTATION_SPEED: f32 = 45.;

#[allow(clippy::type_complexity)]
pub fn rotate(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut world_or_map_query: Query<
        (Option<&ChildOf>, Option<&TiledMapMarker>, &mut Transform),
        Or<(With<TiledMapMarker>, With<TiledWorldMarker>)>,
    >,
) {
    for (parent, map_marker, mut transform) in world_or_map_query.iter_mut() {
        // If we have a map with a parent entity, it probably means this map belongs to a world
        // and we should rotate the world instead of the map
        if parent.is_some() && map_marker.is_some() {
            continue;
        }
        if keyboard_input.pressed(KeyCode::KeyQ) {
            transform.rotate_z(f32::to_radians(ROTATION_SPEED * time.delta_secs()));
        }

        if keyboard_input.pressed(KeyCode::KeyE) {
            transform.rotate_z(f32::to_radians(-(ROTATION_SPEED * time.delta_secs())));
        }
    }
}
