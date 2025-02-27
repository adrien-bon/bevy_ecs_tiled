// This example shows how to use Rapier physics backend.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_rapier2d::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // Examples helper plugins: for this example, contains the logic to switch between maps
        .add_plugins(helper::HelperPlugin)
        // bevy_ecs_tiled main plugin
        .add_plugins(TiledMapPlugin::default())
        // bevy_ecs_tiled physics plugin: this is where we select which physics backend to use
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsRapierBackend>::default())
        // Rapier physics plugins
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, switch_map)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let mut mgr = helper::assets::AssetsManager::new(&mut commands);
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/finite.tmx",
        "A finite orthogonal map with all colliders",
        |c| {
            c.insert((
                TiledMapAnchor::Center,
                TiledPhysicsSettings::<TiledPhysicsRapierBackend>::default(),
            ));
        },
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/infinite.tmx",
        "An infinite orthogonal map with all colliders",
        |c| {
            c.insert((
                TiledMapAnchor::Center,
                TiledPhysicsSettings::<TiledPhysicsRapierBackend>::default(),
            ));
        },
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/finite.tmx",
        "A finite orthogonal map with only tiles colliders named 'collision'",
        |c| {
            c.insert((
                TiledMapAnchor::Center,
                TiledPhysicsSettings::<TiledPhysicsRapierBackend> {
                    objects_layer_filter: TiledName::None,
                    tiles_objects_filter: TiledName::Names(vec!["collision".to_string()]),
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
                TiledMapAnchor::Center,
                TiledPhysicsSettings::<TiledPhysicsRapierBackend> {
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
