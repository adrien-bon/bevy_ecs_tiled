// This example shows Avian2D physics backend with various map orientation.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

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
        // bevy_ecs_tiled physics plugin: this is where we select which physics backend to use
        // Here we use the provided Avian backend to automatically spawn colliders
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default())
        // Avian physics plugins
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .add_plugins(PhysicsDebugPlugin::default())
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, switch_map)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let default_callback: helper::assets::MapInfosCallback = |c| {
        c.insert((
            TilemapAnchor::Center,
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

    // The `helper::AssetsManager` struct is an helper to easily switch between maps in examples.
    // You should NOT use it directly in your games.
    let mut mgr = helper::assets::AssetsManager::new(&mut commands);
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/finite.tmx",
        "A finite orthogonal map",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/infinite.tmx",
        "An infinite orthogonal map",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/hexagonal/finite_flat_top_even.tmx",
        "A finite flat-top (stagger axis = X) hexagonal map with 'even' stagger index",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/hexagonal/finite_flat_top_odd.tmx",
        "A finite flat-top (stagger axis = X) hexagonal map with 'odd' stagger index",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/hexagonal/finite_pointy_top_even.tmx",
        "A finite pointy-top (stagger axis = Y) hexagonal map with 'even' stagger index",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/hexagonal/finite_pointy_top_odd.tmx",
        "A finite pointy-top (stagger axis = Y) hexagonal map with 'odd' stagger index",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/hexagonal/infinite_flat_top_even.tmx",
        "An infinite flat-top (stagger axis = X) hexagonal map with 'even' stagger index",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/hexagonal/infinite_flat_top_odd.tmx",
        "An infinite flat-top (stagger axis = X) hexagonal map with 'odd' stagger index",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/hexagonal/infinite_pointy_top_even.tmx",
        "An infinite pointy-top (stagger axis = Y) hexagonal map with 'even' stagger index",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/hexagonal/infinite_pointy_top_odd.tmx",
        "An infinite pointy-top (stagger axis = Y) hexagonal map with 'odd' stagger index",
        default_callback,
    ));
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
