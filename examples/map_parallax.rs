//! This example demonstrates parallax scrolling with Tiled maps.
//!
//! Parallax scrolling creates a sense of depth by making background layers move
//! slower than foreground layers when the camera moves.
//!
//! ## Setting up parallax in Tiled:
//! 1. Open your map in Tiled
//! 2. Select a layer you want to apply parallax to
//! 3. In the layer properties, set:
//!    - "Parallax X": The horizontal parallax factor (0.0 = fixed, 1.0 = normal speed)
//!    - "Parallax Y": The vertical parallax factor (0.0 = fixed, 1.0 = normal speed)
//!
//! ## Controls:
//! - Arrow keys or WASD: Move camera
//! - Mouse wheel: Zoom in/out
//!
//! The parallax effect will be visible when you move the camera around.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins: prevent blur effect by changing default sampling
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin
        .add_plugins(TiledPlugin::default())
        // Examples helper plugins for camera movement
        .add_plugins(helper::HelperPlugin)
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera and mark it as the parallax camera
    commands.spawn((Camera2d, TiledParallaxCamera));

    // Load and spawn a map
    // Note: For this example to work properly, you need a map with layers that have
    // parallax properties set in Tiled. If your map doesn't have parallax layers,
    // the example will still work but won't show the parallax effect.
    commands.spawn((
        TiledMap(asset_server.load("maps/orthogonal/finite_parallax.tmx")),
        TilemapAnchor::Center,
    ));
}
