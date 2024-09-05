//! This example shows a finite orthogonal map with an external tileset and an example of custom physics.

use std::sync::Arc;

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::ObjectData;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(helper::HelperPlugin)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<TiledMap> = asset_server.load("finite.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        tiled_settings: TiledMapSettings {
            // Enable a custom physics handler to create our
            // tile and object colliders:
            physics_backend: PhysicsBackend::Custom(Arc::new(Box::new(CustomPhysics {}))),
            // This is the default, but we're setting it explicitly here for clarity.
            collision_object_names: ObjectNames::All,
            // Not related to current example, but center the map
            map_positioning: MapPositioning::Centered,
            ..default()
        },
        ..Default::default()
    });
}

// This is our custom collider component
#[derive(Component)]
struct Collider;

// This is struct that we will implement the HandleColliders trait for
struct CustomPhysics;

impl HandleColliders for CustomPhysics {
    fn insert_colliders_from_shapes<'a>(
        &self,
        commands: &'a mut Commands,
        parent_entity: Entity,
        _map_type: &TilemapType,
        grid_size: Option<&TilemapGridSize>,
        object_data: &ObjectData,
    ) -> Option<EntityCommands<'a>> {
        // This basic example will just draw an "X" on the location
        // where a collider should appear.
        let rot = object_data.rotation;
        let pos = match &object_data.shape {
            tiled::ObjectShape::Rect { width, height } => {
                // The origin is the top-left corner of the rectangle when not rotated.
                let pos = Vec2::new(width / 2., -height / 2.);
                pos
            }
            tiled::ObjectShape::Ellipse { width, height } => {
                let pos = Vec2::new(width / 2., -height / 2.);
                pos
            }
            tiled::ObjectShape::Polyline { points } => Vec2::ZERO,
            tiled::ObjectShape::Polygon { points } => Vec2::ZERO,
            _ => {
                return None;
            }
        };

        let mut translation = Vec3::default();
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

        // Construct a custom collider instance. In a real game, you would
        // probably want to use a more complex collider type.
        // Text with one section
        commands.spawn((
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "X",
                TextStyle {
                    font_size: 15.0,
                    ..default()
                },
            ) // Set the justification of the Text
            .with_text_justify(JustifyText::Center)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(transform.translation.y),
                right: Val::Px(transform.translation.x),
                ..default()
            }),
            Collider,
        ));

        let collider = Collider {};
        let mut entity_commands = commands.spawn(collider);
        entity_commands
            .insert(
                TextBundle::from_section("X", TextStyle { ..default() }).with_style(Style {
                    bottom: Val::Px(5.0),
                    right: Val::Px(5.0),
                    ..default()
                }),
            )
            .insert(TransformBundle::from_transform(transform))
            .insert(Name::new(format!("Collider({})", object_data.name)))
            .set_parent(parent_entity);
        Some(entity_commands)
    }
}
