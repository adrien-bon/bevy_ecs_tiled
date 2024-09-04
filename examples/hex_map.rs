//! This example cycle through all four kinds of hexagonal maps and display debug informations about Tiled objects.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        // Enable debug informations about Tiled objects
        .add_plugins(TiledMapDebugPlugin::default())
        .add_plugins(helper::HelperPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, switch_map)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut mgr = helper::assets::AssetsManager::new(&mut commands);
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        TilemapRenderSettings::default(),
        TiledMapSettings {
            map_positioning: MapPositioning::Centered,
            ..Default::default()
        },
        "hex_map_flat_top_even.tmx",
        "A finite flat-top (stagger axis = X) hexagonal map with 'even' stagger index",
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        TilemapRenderSettings::default(),
        TiledMapSettings {
            map_positioning: MapPositioning::Centered,
            ..Default::default()
        },
        "hex_map_flat_top_odd.tmx",
        "A finite flat-top (stagger axis = X) hexagonal map with 'odd' stagger index",
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        TilemapRenderSettings::default(),
        TiledMapSettings {
            map_positioning: MapPositioning::Centered,
            ..Default::default()
        },
        "hex_map_pointy_top_even.tmx",
        "A finite pointy-top (stagger axis = Y) hexagonal map with 'even' stagger index",
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        TilemapRenderSettings::default(),
        TiledMapSettings {
            map_positioning: MapPositioning::Centered,
            ..Default::default()
        },
        "hex_map_pointy_top_odd.tmx",
        "A finite pointy-top (stagger axis = Y) hexagonal map with 'odd' stagger index",
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
