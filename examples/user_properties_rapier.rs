//! This example shows how to map custom tiles / objects properties from Tiled to Bevy Components and manually spawn Rapier colliders from them.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(helper::HelperPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, display_custom_tiles)
        .add_systems(Update, display_objects)
        // We need to register the objects we created in Tiled.
        // Any property which is not registered will not be added to the object entity
        .register_tiled_custom_tile::<TileComponent>("TileComponent")
        .register_tiled_object::<SpawnBundle>("SpawnBundle")
        .register_tiled_object::<ColliderComponent>("ColliderComponent")
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<TiledMap> = asset_server.load("colliders_and_user_properties.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        tiled_settings: TiledMapSettings {
            // Disable colliders automatic spawn on all objects layers / all tiles with collision objects
            // We will only spawn colliders for objects / custom tiles we want
            // If you check the .tmx file, you will see that the 'SpawnBundle'
            // object will not have an associated collider
            collision_layer_names: ObjectNames::None,
            collision_object_names: ObjectNames::None,
            // Not related to current example, but center the map
            map_positioning: MapPositioning::Centered,
            ..default()
        },
        ..Default::default()
    });
}

// Only print the first tile to avoid flooding the console
fn display_custom_tiles(q_tile: Query<(&TilePos, &TileComponent)>) {
    for (position, tile_component) in q_tile.iter() {
        info_once!(
            "Found TileComponent [{:?} @ {:?}]",
            tile_component,
            position
        );
    }
}

// Only print the first objects to avoid flooding the console
fn display_objects(q_object: Query<(&Transform, Option<&SpawnInfos>, Option<&ColliderComponent>)>) {
    for (transform, spawn_infos, collider_comp) in q_object.iter() {
        if let Some(i) = spawn_infos {
            info_once!("Found SpawnInfos [{:?} @ {:?}]", i, transform);
        }
        if let Some(c) = collider_comp {
            info_once!("Found ColliderComponent [{:?} @ {:?}]", c, transform);
        }
    }
}

// Custom properties definition: see ./examples/user_properties.rs for an
// in-depth explanation of how to declare them. Here we will only focus on the
// way we can associate Rapier colliders to them.
// The important part here is to add a 'tiled_observer' to our structs
#[derive(TiledObject, Bundle, Default, Reflect, Debug)]
struct SpawnBundle {
    infos: SpawnInfos,
}

#[derive(TiledObject, Component, Default, Reflect, Debug)]
#[tiled_observer(collider_component_observer)]
struct ColliderComponent {
    #[tiled_rename = "DamagePerSecond"]
    damage_per_second: f32,
    is_visible: bool,
}

#[derive(TiledCustomTile, Component, Default, Debug, Reflect)]
#[tiled_observer(tile_component_observer)]
struct TileComponent {
    prefered_color: bevy::color::Color,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
struct SpawnInfos {
    #[tiled_rename = "Type"]
    ty: SpawnType,
}

#[derive(TiledEnum, Default, Reflect, Debug)]
enum SpawnType {
    #[default]
    Unknown,
    Player,
}

// Here are a custom tile / an object observer:
// we just call the 'spawn_rapier_collider' on the event
// to automatically spawn associated rapier collider
// We can eventually provide a 'ColliderCallback' to
// add more components to the collider
fn collider_component_observer(trigger: Trigger<TiledObjectCreated>, commands: Commands) {
    trigger.event().spawn_rapier_collider(commands, |_| {});
}

// Special case for tiles: you can filter for which
// associated collision objects you actually want
// to spawn a collider
fn tile_component_observer(trigger: Trigger<TiledCustomTileCreated>, commands: Commands) {
    trigger.event().spawn_rapier_collider(
        commands,
        // We will ignore collision objects not named 'collision'
        // (notice the black tiles don't have a collider)
        ObjectNames::Names(vec!["collision".to_string()]),
        |_| {},
    );
}
