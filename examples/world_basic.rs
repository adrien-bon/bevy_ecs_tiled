// This example shows the basic usage of the plugin to load a Tiled world.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // bevy_ecs_tiled main plugin
        .add_plugins(TiledMapPlugin::default())
        // Examples helper plugins: for this example, contains the logic to move the camera
        .add_plugins(helper::HelperPlugin)
        // Add our systems and run the app!
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera (required by Bevy)
    commands.spawn(Camera2d);

    // Load the world ...
    let world_handle: Handle<TiledWorld> = asset_server.load("worlds/orthogonal.world");

    // ... then spawn it !
    let mut world_entity = commands.spawn(TiledWorldHandle(world_handle));

    // You can eventually add some extra settings to your world
    world_entity.insert((TiledMapAnchor::Center, TiledWorldChunking::new(200., 200.)));
}
