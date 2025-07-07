// This example shows how to load Tiled World files and demonstrates chunking the loaded maps.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

const STEP_SIZE: u32 = 8;

fn main() {
    App::new()
        // Bevy default plugins: prevent blur effect by changing default sampling
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done
        .add_plugins(TiledPlugin::default())
        // Examples helper plugins, such as the logic to pan and zoom the camera
        // This should not be used directly in your game (but you can always have a look)
        .add_plugins(helper::HelperPlugin)
        // Add bevy_ecs_tiled debug plugins
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

    // Load and spawn the world
    commands.spawn((
        TiledWorld(asset_server.load("worlds/orthogonal.world")),
        TilemapAnchor::Center,
        TiledWorldChunking::new(200., 200.),
    ));

    // Add a helper text to display current chunking values
    commands
        .spawn((
            Text::new("[+/-] Chunking: "),
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

fn input(mut chunking: Query<&mut TiledWorldChunking>, keys: Res<ButtonInput<KeyCode>>) {
    let Ok(mut chunking) = chunking.single_mut() else {
        return;
    };

    if keys.pressed(KeyCode::Minus) {
        // Decrease the chunking size
        if let Some(c) = chunking.0 {
            if (c.x - STEP_SIZE as f32) > 0. {
                *chunking = TiledWorldChunking::new(c.x - STEP_SIZE as f32, c.y - STEP_SIZE as f32);
            }
        }
    }

    if keys.pressed(KeyCode::Equal) {
        if let Some(c) = chunking.0 {
            if c.x < f32::MAX - STEP_SIZE as f32 {
                *chunking = TiledWorldChunking::new(c.x + STEP_SIZE as f32, c.y + STEP_SIZE as f32);
            }
        }
    }
}

fn text_update_system(
    chunking: Query<&TiledWorldChunking>,
    mut query: Query<&mut TextSpan, With<HelperText>>,
) {
    let Ok(chunking) = chunking.single() else {
        return;
    };

    for mut span in &mut query {
        span.0 = chunking.0.map_or(String::from("None"), |chunking| {
            format!("{}x{}", chunking.x, chunking.y)
        });
    }
}
