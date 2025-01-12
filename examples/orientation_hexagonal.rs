//! This example cycles through different kinds of hexagonal maps.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // Examples helper plugins: for this example, contains the logic to switch between maps
        .add_plugins(helper::HelperPlugin)
        // bevy_ecs_tilemap and bevy_ecs_tiled main plugins
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin::default())
        // Enable debug informations
        .add_plugins(TiledDebugPluginGroup)
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, switch_map)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let default_callback: helper::assets::MapInfosCallback = |c| {
        c.insert(TiledMapSettings {
            layer_positioning: LayerPositioning::Centered,
            ..default()
        });
    };

    let mut mgr = helper::assets::AssetsManager::new(&mut commands);
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "hex_map_flat_top_even.tmx",
        "A finite flat-top (stagger axis = X) hexagonal map with 'even' stagger index",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "hex_map_flat_top_odd.tmx",
        "A finite flat-top (stagger axis = X) hexagonal map with 'odd' stagger index",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "hex_map_pointy_top_even.tmx",
        "A finite pointy-top (stagger axis = Y) hexagonal map with 'even' stagger index",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "hex_map_pointy_top_odd.tmx",
        "A finite pointy-top (stagger axis = Y) hexagonal map with 'odd' stagger index",
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
