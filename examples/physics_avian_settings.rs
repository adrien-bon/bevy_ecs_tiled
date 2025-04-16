// This example shows how to use Avian2D physics backend.

use avian2d::prelude::*;
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

    // The `helper::AssetsManager` struct is an helper to easily switch between maps in examples.
    // You should NOT use it directly in your games.
    let mut mgr = helper::assets::AssetsManager::new(&mut commands);
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/finite.tmx",
        "A finite orthogonal map with all colliders",
        |c| {
            c.insert((
                TilemapAnchor::Center,
                TiledPhysicsSettings::<TiledPhysicsAvianBackend>::default(),
            ));
        },
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/infinite.tmx",
        "An infinite orthogonal map with all colliders",
        |c| {
            c.insert((
                TilemapAnchor::Center,
                TiledPhysicsSettings::<TiledPhysicsAvianBackend>::default(),
            ));
        },
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/finite.tmx",
        "A finite orthogonal map with only tiles colliders named 'collision'",
        |c| {
            c.insert((
                TilemapAnchor::Center,
                TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
                    objects_layer_filter: TiledName::None,
                    tiles_objects_filter: TiledName::Names(vec![String::from("collision")]),
                    ..default()
                },
            ));
        },
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/finite.tmx",
        "A finite orthogonal map with only object colliders",
        |c| {
            c.insert((
                TilemapAnchor::Center,
                TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
                    objects_layer_filter: TiledName::All,
                    tiles_objects_filter: TiledName::None,
                    ..default()
                },
            ));
        },
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
