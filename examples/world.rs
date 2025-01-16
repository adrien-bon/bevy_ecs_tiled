// This example shows how to load a Tiled World and demonstrates
// dynamic loading of the world maps for performance

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

const STEP_SIZE: u32 = 8;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // bevy_ecs_tilemap and bevy_ecs_tiled main plugins
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin::default())
        // Examples helper plugins: for this example, contains the logic to move the camera
        .add_plugins(helper::HelperPlugin)
        // Enable debug informations
        .add_plugins(TiledDebugPluginGroup)
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
            chunking: Some(Vec2::new(200., 200.)),
        },
    ));

    commands
        .spawn((
            Text::new("[W/A/S/D] Pan [Z/X] Zoom [+/-] Chunking: "),
            TextFont {
                font_size: 24.,
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextFont {
                font_size: 24.,
                ..default()
            },
            HelperText,
        ));
}

fn input(mut settings: Query<&mut TiledWorldSettings>, keys: Res<ButtonInput<KeyCode>>) {
    let Ok(mut settings) = settings.get_single_mut() else {
        return;
    };

    if keys.pressed(KeyCode::Minus) {
        // Decrease the chunking size
        if let Some(chunking) = settings.chunking {
            if (chunking.x - STEP_SIZE as f32) > 0. {
                settings.chunking = Some(Vec2::new(
                    chunking.x - STEP_SIZE as f32,
                    chunking.y - STEP_SIZE as f32,
                ));
            }
        }
    }

    if keys.pressed(KeyCode::Equal) {
        if let Some(chunking) = settings.chunking {
            if chunking.x < f32::MAX - STEP_SIZE as f32 {
                settings.chunking = Some(Vec2::new(
                    chunking.x + STEP_SIZE as f32,
                    chunking.y + STEP_SIZE as f32,
                ));
            }
        }
    }
}

fn text_update_system(
    settings: Query<&TiledWorldSettings>,
    mut query: Query<&mut TextSpan, With<HelperText>>,
) {
    let Ok(settings) = settings.get_single() else {
        return;
    };

    for mut span in &mut query {
        span.0 = settings.chunking.map_or(String::from("None"), |chunking| {
            format!("{}x{}", chunking.x, chunking.y)
        });
    }
}
