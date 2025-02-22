//! This example cycles through different kinds of isometric maps.

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
        // bevy_ecs_tiled main plugin
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
        c.insert((
            TiledMapSettings::with_layer_positioning(LayerPositioning::Centered),
            // For isometric maps, it can be useful to tweak bevy_ecs_tilemap render settings.
            // TilemapRenderSettings provide the 'y_sort' parameter to sort chunks using their y-axis
            // position during rendering.
            // However, it applies to whole chunks, not individual tile, so we have to force the chunk
            // size to be exactly one tile along the y-axis
            TilemapRenderSettings {
                render_chunk_size: UVec2::new(64, 1),
                y_sort: true,
            },
        ));
    };

    let mut mgr = helper::assets::AssetsManager::new(&mut commands);
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/isometric/finite_diamond.tmx",
        "A finite 'diamond' isometric map",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/isometric/infinite_diamond.tmx",
        "An infinite 'diamond' isometric map",
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
