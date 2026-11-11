//! This example cycles through Tiled worlds made of maps with different orientations.

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
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, switch_world)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let default_callback: helper::assets::MapInfosCallback = |c| {
        c.insert(TilemapAnchor::Center);
    };
    let isometric_callback: helper::assets::MapInfosCallback = |c| {
        c.insert((
            TilemapAnchor::Center,
            // For isometric maps, it can be useful to tweak `bevy_ecs_tilemap` render settings.
            // [`TilemapRenderSettings`] provides the `y_sort`` parameter to sort chunks using their y-axis
            // position during rendering.
            // However, it applies to whole chunks, not individual tile, so we have to force the chunk
            // size to be exactly one tile along the y-axis.
            // These settings are applied to all the maps of the world.
            TilemapRenderSettings {
                render_chunk_size: UVec2::new(64, 1),
                y_sort: true,
            },
        ));
    };

    // The `helper::AssetsManager` struct is an helper to easily switch between maps in examples.
    // You should NOT use it directly in your games.
    let mut mgr = helper::assets::AssetsManager::new(&mut commands);
    mgr.add_map(helper::assets::MapInfos::new_world(
        &asset_server,
        "worlds/orthogonal.world",
        "A world made of 5x5 orthogonal maps",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new_world(
        &asset_server,
        "worlds/isometric.world",
        "A world made of 2x2 isometric (diamond) maps",
        isometric_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new_world(
        &asset_server,
        "worlds/hexagonal_pointy_top.world",
        "A world made of 2x2 pointy-top (stagger axis = Y) hexagonal maps with 'odd' stagger index",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new_world(
        &asset_server,
        "worlds/hexagonal_flat_top.world",
        "A world made of 2x2 flat-top (stagger axis = X) hexagonal maps with 'even' stagger index",
        default_callback,
    ));
    commands.insert_resource(mgr);
}

fn switch_world(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mgr: ResMut<helper::assets::AssetsManager>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        mgr.cycle_map(&mut commands);
    }
}
