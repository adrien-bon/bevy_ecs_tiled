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
        .add_plugins(helper::HelperPlugin)
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
            chunking: Some((200, 200)),
        },
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

fn input(mut settings: Query<&mut TiledWorldSettings>, keys: Res<ButtonInput<KeyCode>>) {
    let mut settings = settings.single_mut();

    if keys.pressed(KeyCode::Minus) {
        // Decrease the chunking size
        if let Some((width, height)) = settings.chunking {
            if (width as i32 - STEP_SIZE as i32) > 0 {
                settings.chunking = Some((width - STEP_SIZE, height - STEP_SIZE));
            }
        }
    }

    if keys.pressed(KeyCode::Equal) {
        if let Some((width, height)) = settings.chunking {
            if width < u32::MAX - STEP_SIZE {
                settings.chunking = Some((width + STEP_SIZE, height + STEP_SIZE));
            }
        }
    }
}

fn text_update_system(
    settings: Query<&TiledWorldSettings>,
    mut query: Query<&mut TextSpan, With<HelperText>>,
) {
    for mut span in &mut query {
        let settings = settings.single();
        span.0 = if let Some((width, height)) = settings.chunking {
            format!("{}x{}", width, height)
        } else {
            "None".to_string()
        };
    }
}
