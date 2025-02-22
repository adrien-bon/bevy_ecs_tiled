//! This example shows the basic usage of the plugin.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // bevy_ecs_tiled main plugin
        .add_plugins(TiledMapPlugin::default())
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera (required by Bevy)
    commands.spawn(Camera2d);

    // Load the map ...
    let map_handle: Handle<TiledMap> = asset_server.load("maps/orthogonal/finite.tmx");

    // ... then spawn it !
    let mut map_entity = commands.spawn(TiledMapHandle(map_handle));

    // You can eventually add some extra settings to your map
    map_entity.insert(TiledMapSettings::with_layer_positioning(
        LayerPositioning::Centered,
    ));
}
