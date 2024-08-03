use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub fn rotate(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut tilemap: Query<(Entity, &mut Transform), With<TiledMapMarker>>,
) {
    for (_, mut transform) in tilemap.iter_mut() {
        if keyboard_input.pressed(KeyCode::KeyQ) {
            transform.rotate_z(f32::to_radians(90.0 * time.delta_seconds()));
        }

        if keyboard_input.pressed(KeyCode::KeyE) {
            transform.rotate_z(f32::to_radians(-90.0 * time.delta_seconds()));
        }
    }
}
