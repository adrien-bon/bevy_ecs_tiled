//! This example shows the basic usage of the plugin.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // bevy_ecs_tilemap and bevy_ecs_tiled main plugins
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin::default())
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera (required by Bevy)
    commands.spawn(Camera2dBundle::default());

    // Load the map ...
    let map_handle: Handle<TiledMap> = asset_server.load("finite.tmx");

    // ... then spawn it !
    let mut map_entity = commands.spawn(TiledMapHandle(map_handle));

    // You can eventually add some extra settings to your map (but it's not needed)
    map_entity.insert((
        TiledMapSettings {
            layer_positioning: LayerPositioning::Centered,
            ..default()
        },
        TilemapRenderSettings::default()
    ));
}
