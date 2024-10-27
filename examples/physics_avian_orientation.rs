// This example shows Avian2D physics backend with various map orientation.

use avian2d::prelude::*;
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
        // bevy_ecs_tiled physics plugin: this is where we select which physics backend to use
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
    commands.spawn(Camera2dBundle::default());

    let default_callback: helper::assets::MapInfosCallback = |c| {
        c.insert(TiledMapSettings {
            layer_positioning: LayerPositioning::Centered,
            ..default()
        });
        c.insert(TilemapRenderSettings {
            render_chunk_size: UVec2::new(1, 1),
            y_sort: true,
        });
    };

    let mut mgr = helper::assets::AssetsManager::new(&mut commands);
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "finite.tmx",
        "A finite orthogonal map",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "infinite.tmx",
        "An infinite orthogonal map",
        default_callback,
    ));
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
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "isometric_diamond_map.tmx",
        "A finite diamond isometric map",
        default_callback,
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "isometric_staggered_map.tmx",
        "A finite staggered isometric map",
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
