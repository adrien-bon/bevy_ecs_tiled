//! This example shows how to map custom tiles and objects properties from Tiled to Bevy Components.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(helper::HelperPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, display_custom_tiles)
        .add_systems(Update, display_objects)
        // We need to register custom tiles / objects we created in Tiled.
        // Any class which is not registered will not be added to the tile / object entity
        .register_tiled_custom_tile::<TileBundle>("TileBundle")
        .register_tiled_custom_tile::<TileComponent>("TileComponent")
        .register_tiled_object::<SpawnBundle>("SpawnBundle")
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<TiledMap> = asset_server.load("hex_map_pointy_top_odd.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

// Only print the first tiles to avoid flooding the console
fn display_custom_tiles(q_tile: Query<(&TilePos, Option<&BiomeInfos>, Option<&TileComponent>)>) {
    for (position, biome_infos, tile_component) in q_tile.iter() {
        if let Some(i) = biome_infos {
            info_once!("Found TileBundle [{:?} @ {:?}]", i, position);
        }
        if let Some(c) = tile_component {
            info_once!("Found TileComponent [{:?} @ {:?}]", c, position);
        }
    }
}

// Only print the first object to avoid flooding the console
fn display_objects(q_object: Query<(&Transform, &SpawnInfos)>) {
    for (transform, spawn_infos) in q_object.iter() {
        info_once!("Found SpawnInfos [{:?} @ {:?}]", spawn_infos, transform);
    }
}

// Custom types definition for custom tiles
//
// They must implement the following traits:
// - 'TiledCustomTile', to declare it as a custom tile and trigger the custom properties mapping
// - 'Bundle' or 'Component', so we can insert it on the tile entity
// - 'Default' trait, in case a property is not set in Tiled
// 'Debug' and 'Reflect' traits are for convenience but are not mandatory
//
// 'TiledCustomTile' struct accept the following attributes:
// - tiled_observer: name of an observer which will be triggered once the tile is actually added to the world
//   Please note that it accepts any arguments compatible with Bevy observers
// - tiled_rename: name of the Tiled type, in case it's different from the structure field
// - tiled_skip: skip the following field and do not try to get it's value from Tiled custom properties.
//   Instead use the struct default value.

#[derive(TiledCustomTile, Bundle, Default, Debug, Reflect)]
struct TileBundle {
    // We expect this field to be named 'BiomeInfos' in Tiled custom properties
    // If a field is not defined in Tiled, we will use the field Rust's default value
    #[tiled_rename = "BiomeInfos"]
    infos: BiomeInfos,
}

#[derive(TiledCustomTile, Component, Default, Debug, Reflect)]
#[tiled_observer(test_tile_observer)]
struct TileComponent {
    prefered_color: bevy::color::Color,
}

// Custom types definition for objects
//
// They must implement the following traits:
// - 'TiledObject', to declare it as an object and trigger the custom properties mapping
// - 'Bundle' or 'Component', so we can insert it on the object entity
// - 'Default' trait, in case a property is not set in Tiled
// 'Debug' and 'Reflect' traits are for convenience but are not mandatory
//
// 'TiledCustomTile' struct accept the following attributes:
// - tiled_observer: name of an observer which will be triggered once the object is actually added to the world
//   Please note that it accepts any arguments compatible with Bevy observers
// - tiled_rename: name of the Tiled type, in case it's different from the structure field
// - tiled_skip: skip the following field and do not try to get it's value from Tiled custom properties

#[derive(TiledObject, Bundle, Default, Reflect, Debug)]
#[tiled_observer(test_object_observer)]
struct SpawnBundle {
    infos: SpawnInfos,
}

// We can also define custom classes
//
// They must implement the following traits:
// - 'TiledClass', to declare it as a custom class and trigger the custom properties mapping
// - 'Component', so we can insert it on the custom tile / object entity
// - 'Default' trait, in case a property is not set in Tiled
// 'Debug' and 'Reflect' traits are for convenience but are not mandatory
//
// 'TiledClass' struct accept the following attributes:
// - tiled_rename: name of the Tiled type, in case it's different from the structure field
// - tiled_skip: skip the following field and do not try to get it's value from Tiled custom properties

#[derive(TiledClass, Component, Default, Debug, Reflect)]
struct BiomeInfos {
    #[tiled_rename = "Type"]
    ty: BiomeType,
    #[tiled_rename = "BlockLineOfSight"]
    block_line_of_sight: bool,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
struct SpawnInfos {
    #[tiled_rename = "Type"]
    ty: SpawnType,
}

// We can also define custom enums
// Please note that only 'string' enum from Tiled are supported
//
// They must implement the following traits:
// - 'TiledEnum', to declare it as a custom enum and trigger the custom properties mapping
// - 'Default' trait, in case a property is not set in Tiled
// 'Debug' and 'Reflect' traits are for convenience but are not mandatory

#[derive(TiledEnum, Default, Reflect, Debug)]
enum BiomeType {
    #[default]
    Unknown,
    Plain,
    Desert,
    Forest,
    Mountain,
}

#[derive(TiledEnum, Default, Reflect, Debug)]
enum SpawnType {
    #[default]
    Unknown,
    Player,
}

// Here are a custom tile / an object observer: they can use all arguments available to observers: Commands, Query, Res, etc...
// Note that trigger.event().entity and trigger.entity() are equivalent
// WARNING: since observers are fired as we load the map, some entity could have been not yet spawned
fn test_tile_observer(
    trigger: Trigger<TiledCustomTileCreated>,
    q_tile: Query<&TileComponent>,
    // following arguments are not used but illustrate what can be used in an observer
    mut _commands: Commands,
    _q_tilemap: Query<&TilemapGridSize>,
    _asset_server: Res<AssetServer>,
) {
    info!("OBSERVER for tile [{:?}]", trigger.entity());
    if let Ok(comp) = q_tile.get(trigger.entity()) {
        info!("(OBSERVER) Found TileComponent for tile: {:?}", comp);
    }
}

fn test_object_observer(trigger: Trigger<TiledObjectCreated>, q_object: Query<&SpawnInfos>) {
    info!("OBSERVER for object [{:?}]", trigger.entity());
    if let Ok(spawn_infos) = q_object.get(trigger.entity()) {
        info!("(OBSERVER) Found SpawnInfos for object: {:?}", spawn_infos);
    }
}
