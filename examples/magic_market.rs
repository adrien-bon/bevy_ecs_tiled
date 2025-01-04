//! This example shows the basic usage of the plugin but load a more complex / realistic map.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins: prevent blur effect by changing default sampling
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // bevy_ecs_tilemap and bevy_ecs_tiled main plugins
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin::default())
        // Examples helper plugins: for this example, will spawn inspector and handle camera
        .add_plugins(helper::HelperPlugin)
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera (required by Bevy)
    commands.spawn(Camera2d);

    // Load and spawn the map
    commands.spawn((
        TiledMapHandle(asset_server.load("Magic Market/Magic Market.tmx")),
        TiledMapSettings {
            layer_positioning: LayerPositioning::Centered,
            ..default()
        },
    ));
}
