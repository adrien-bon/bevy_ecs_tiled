// This example shows how to load a Tiled World and demonstrates
// dynamic loading of the world maps for performance

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tiled::prelude::*;

const STEP_SIZE: u32 = 8;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // bevy_ecs_tilemap and bevy_ecs_tiled main plugins
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin::default())
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, (input, text_update_system))
        .run();
}

#[derive(Component, Debug)]
struct HelperText;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera (required by Bevy)
    commands.spawn(Camera2d);

    // Load the world ...
    let world_handle: Handle<TiledWorld> = asset_server.load("world/world.world");

    // ... then spawn it !
    let mut world_entity = commands.spawn(TiledWorldHandle(world_handle));

    // You can eventually add some extra settings to your world
    world_entity.insert((
        TiledMapSettings {
            layer_positioning: LayerPositioning::Centered,
            ..default()
        },
        TiledWorldSettings {
            chunking: true,
            chunking_width: 200,
            chunking_height: 200,
        },
        TilemapRenderSettings::default(),
    ));

    commands
        .spawn((
            Text::new("[W/A/S/D] Pan [Z/X] Zoom [+/-] Chunking: "),
            TextFont {
                font_size: 24.0,
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            HelperText,
        ));

}

fn input(
    mut query: Query<(&Camera2d, &mut Transform)>,
    mut settings: Query<&mut TiledWorldSettings>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut settings = settings.single_mut();

    for (_, mut transform) in query.iter_mut() {
        let mut translation = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            translation.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            translation.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            translation.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            translation.x += 1.0;
        }

        // Zoom up and down
        if keys.pressed(KeyCode::KeyZ) {
            transform.scale *= 1.02;
        }
        if keys.pressed(KeyCode::KeyX) {
            transform.scale /= 1.02;
        }

        if keys.pressed(KeyCode::Minus) {
            // Decrease the chunking size
            if (settings.chunking_width as i32 - STEP_SIZE as i32) > 0 {
                settings.chunking_width -= STEP_SIZE;
                settings.chunking_height -= STEP_SIZE;

                log::info!(
                    "Chunking size: {}x{}",
                    settings.chunking_width,
                    settings.chunking_height
                );
            }
        }

        if keys.pressed(KeyCode::Equal) {
            // Increase the chunking size
            if settings.chunking_width < u32::MAX - STEP_SIZE {
                settings.chunking_width += STEP_SIZE;
                settings.chunking_height += STEP_SIZE;

                log::info!(
                    "Chunking size: {}x{}",
                    settings.chunking_width,
                    settings.chunking_height
                );
            }

        }

        transform.translation += translation * 5.0;
    }
}

fn text_update_system(
    settings: Query<&TiledWorldSettings>,
    mut query: Query<&mut TextSpan, With<HelperText>>,
) {
    for mut span in &mut query {
        let settings = settings.single();

        **span = format!(
            "{}x{}",
            settings.chunking_width, settings.chunking_height
        );
    }
}
