//! This example cycles through maps created with different tilesets that use spacings and margins.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins: prevent blur effect by changing default sampling
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done
        .add_plugins(TiledMapPlugin::default())
        // Examples helper plugins, such as the logic to pan and zoom the camera
        // This should not be used directly in your game (but you can always have a look)
        .add_plugins(helper::HelperPlugin)
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

    // Spacing and margin are read from the tsx file and applied automatically.
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/spacing/no-spacing.tmx",
        "Tileset without spacing and margin",
        default_callback,
    ));

    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/spacing/8px-spacing-8px-margin.tmx",
        "Tileset with 8px spacing and 8px margin",
        default_callback,
    ));

    // Issues arise when spacing and margins differ.
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/spacing/8px-spacing.tmx",
        "Tileset with 8px spacing and no margin",
        default_callback,
    ));

    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/spacing/8px-margin.tmx",
        "Tileset with 8px margin an no spacing",
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
