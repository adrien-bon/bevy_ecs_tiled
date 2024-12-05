//! This example cycles through different map settings that can be applied.

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
        "finite.tmx",
        "A map using default settings",
        |_| {},
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "finite.tmx",
        "A map using LayerPositioning::Centered",
        |c| {
            c.insert(TiledMapSettings {
                layer_positioning: LayerPositioning::Centered,
                ..default()
            });
        },
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "finite.tmx",
        "A map using LayerPositioning::TiledOffset",
        |c| {
            c.insert(TiledMapSettings {
                layer_positioning: LayerPositioning::TiledOffset,
                ..default()
            });
        },
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "finite.tmx",
        "A map using an initial Transform (rotation = 45°)",
        |c| {
            // You can directly insert a Transform to the entity holding the map
            c.insert(Transform::from_rotation(Quat::from_rotation_z(45.)));
        },
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "finite.tmx",
        "A map using an initial Visibility (hidden)",
        |c| {
            // You can directly insert a Visibility to the entity holding the map
            c.insert(Visibility::Hidden);
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
