use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

const ROTATION_SPEED: f32 = 45.;

#[cfg(feature = "world")]
type MapOrWorld = Or<(With<TiledMap>, With<TiledWorld>)>;
#[cfg(not(feature = "world"))]
type MapOrWorld = With<TiledMap>;

pub fn rotate(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut world_or_map_query: Query<
        (Option<&ChildOf>, Option<&TiledMap>, &mut Transform),
        MapOrWorld,
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
