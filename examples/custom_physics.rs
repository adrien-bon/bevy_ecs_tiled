//! This example shows how to use a custom physics backend.

use bevy::{
    color::palettes::css::{PURPLE, RED},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(helper::HelperPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, handle_colliders_creation_event)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<TiledMap> = asset_server.load("finite.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        tiled_settings: TiledMapSettings {
            // Enable a custom physics handler to create our
            // tile and object colliders: you need to add a system
            // which receive `CustomColliderCreationEvent` events
            physics_backend: PhysicsBackend::Custom,
            // This is the default, but we're setting it explicitly here for clarity.
            collision_object_names: ObjectNames::All,
            // Not related to current example, but center the map
            map_positioning: MapPositioning::Centered,
            ..default()
        },
        ..Default::default()
    });
}

// System responsible for spawning custom colliders:
// this simple example will spawn a 2D mesh at the center of the collider.
// Collider entity is already spawned and is available directly in the event
fn handle_colliders_creation_event(
    mut commands: Commands,
    mut ev_custom_collider_created: EventReader<CustomColliderCreationEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for ev in ev_custom_collider_created.read() {
        let rot = ev.object_data.rotation;
        let pos = match &ev.object_data.shape {
            tiled::ObjectShape::Rect { width, height } => {
                // The origin is the top-left corner of the rectangle when not rotated.
                Vec2::new(width / 2., -height / 2.)
            }
            tiled::ObjectShape::Ellipse { width, height } => Vec2::new(width / 2., -height / 2.),
            tiled::ObjectShape::Polyline { points: _ } => Vec2::ZERO,
            tiled::ObjectShape::Polygon { points: _ } => Vec2::ZERO,
            _ => {
                return;
            }
        };

        // Use a PURPLE color for objects colliders
        let mut color = Color::from(PURPLE);

        let mut translation = Vec3::default();
        // If we have a grid_size, it means we are adding colliders for a tile:
        // we need to take into account object position, which are relative to the tile
        // If we don't have a grid_size, it means we are adding colliders for a standalone object
        // we need to ignore object position, since our parent should already have the correct position
        if let Some(grid_size) = ev.grid_size {
            translation = Vec3::new(
                ev.object_data.x - grid_size.x / 2.,
                (grid_size.y - ev.object_data.y) - grid_size.y / 2.,
                0.,
            );
            // Use a RED color for tiles colliders
            color = Color::from(RED);
        }

        let transform = Transform {
            translation,
            rotation: Quat::from_rotation_z(f32::to_radians(-rot)),
            ..default()
        } * Transform::from_translation(Vec3::new(pos.x, pos.y, 5.)); // Add a small Z offset to make sure we will see our mesh above tiles

        commands
            .entity(ev.collider_entity)
            .insert(MaterialMesh2dBundle {
                mesh: meshes.add(Rectangle::default()).into(),
                transform: transform.with_scale(Vec3::splat(10.)),
                material: materials.add(color),
                ..default()
            });
    }
}
