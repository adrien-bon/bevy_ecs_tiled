//! This example shows how to map custom tiles / objects properties from Tiled to Bevy Components and manually spawn Avian colliders from them.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy::ecs::reflect::ReflectBundle;
use bevy::ecs::reflect::ReflectComponent;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(helper::HelperPlugin)
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .add_plugins(PhysicsDebugPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, display_custom_tiles)
        .add_systems(Update, display_objects)
        // We need to register the objects we created in Tiled.
        // Any property which is not registered will not be added to the object entity
        // .register_tiled_custom_tile::<TileComponent>("TileComponent")
        .register_type::<SpawnBundle>()
        .register_type::<ColliderComponent>()
        .register_type::<TileComponent>()
        .register_type::<SpawnInfos>()
        .register_type::<SpawnType>()
        // .register_tiled_object::<SpawnBundle>("SpawnBundle")
        // .register_tiled_object::<ColliderComponent>("ColliderComponent")
        .observe(load_object_properties)
        .observe(handle_event_2)
        .run();
}

fn load_object_properties(
    trigger: Trigger<TiledObjectCreated>,
    q_storage: Query<&TiledIdStorage>,
    q_name: Query<Option<&Name>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let storage = q_storage.get(event.map)
        .expect("map missing");
    
    // let mut commands = commands.entity(event.object);
    
    for &e in storage.objects.values() {
        println!("{:?}", q_name.get(e));
    }
}



fn handle_event_2(trigger: Trigger<TiledObjectCreated>) {
    // println!("b");
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
fn display_custom_tiles(
    q_tile: Query<(&TilePos, &TileComponent)>,
    q_map: Query<(&TiledIdStorage, Option<&Name>)>,
    q_name: Query<&Name>,
) {
    // for (store, name) in q_map.iter() {
    //     if let Some(name) = name {
    //         println!("{}", name);
    //     }
    //     for (&id, &entity) in store.storage.iter() {
    //         // println!("  {id}: {:?}", q_name.get(entity));
    //     }
    // }
    
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
#[derive(Bundle, Default, Reflect, Debug)]
#[reflect(Bundle)]
struct SpawnBundle {
    infos: SpawnInfos,
    collider: ColliderComponent,
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component)]
struct ColliderComponent {
    damage_per_second: f32,
    is_visible: bool,
}

#[derive(TiledCustomTile, Component, Default, Debug, Reflect)]
#[reflect(Component)]
struct TileComponent {
    prefered_color: bevy::color::Color,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
#[reflect(Component)]
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
// we just call the 'spawn_collider' on the event
// to automatically spawn associated rapier collider
// We can eventually provide a 'ColliderCallback' to
// add more components to the collider
fn collider_component_observer(trigger: Trigger<TiledObjectCreated>, commands: Commands) {
    dbg!();
    trigger.event().spawn_collider(commands, |_| {});
}

// Special case for tiles: you can filter for which
// associated collision objects you actually want
// to spawn a collider
fn tile_component_observer(trigger: Trigger<TiledCustomTileCreated>, commands: Commands) {
    dbg!();
    trigger.event().spawn_collider(
        commands,
        // We will ignore collision objects not named 'collision'
        // (notice the black tiles don't have a collider)
        ObjectNames::Names(vec!["collision".to_string()]),
        |_| {},
    );
}
