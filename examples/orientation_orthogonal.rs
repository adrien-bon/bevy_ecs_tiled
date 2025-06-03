//! This example cycles through different kinds of orthogonal maps.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

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
        .add_systems(Update, switch_map)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let default_callback: helper::assets::MapInfosCallback = |c| {
        c.insert(TilemapAnchor::Center);
    };

    // The `helper::AssetsManager` struct is an helper to easily switch between maps in examples.
    // You should NOT use it directly in your games.
    let mut mgr = helper::assets::AssetsManager::new(&mut commands);
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/finite.tmx",
        "A finite orthogonal map with a single external tileset",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/finite_embedded.tmx",
        "A finite orthogonal map with a single embedded tileset",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/infinite.tmx",
        "An infinite orthogonal map with a single external tileset",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/infinite_embedded.tmx",
        "An infinite orthogonal map with a single embedded tileset",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        // For simplicity sake, we use two tilesets which actually use the same images
        // However, we can verify with the inspector that the map actually use tiles
        // from both tilesets
        "maps/orthogonal/multiple_tilesets.tmx",
        "A finite orthogonal map with multiple external tilesets",
        default_callback,
    ));
    commands.insert_resource(mgr);
}

fn switch_map(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mgr: ResMut<helper::assets::AssetsManager>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        mgr.cycle_map(&mut commands);
    }
}
