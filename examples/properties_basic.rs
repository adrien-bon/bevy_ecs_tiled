//! This example shows how to map custom tiles and objects properties from Tiled to Bevy Components.

use std::env;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

fn main() {
    // Use a custom file path to export registered types in Tiled format
    let mut path = env::current_dir().unwrap();
    path.push("examples");
    path.push("properties_basic.json");

    App::new()
        // Bevy default plugins: prevent blur effect by changing default sampling
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done
        // For demonstration purpose, provide a custom path where to export registered types
        .add_plugins(TiledPlugin(TiledPluginConfig {
            // Note: if you set this setting to `None`
            // properties won't be exported anymore but
            // you will still be able to load them from the map
            tiled_types_export_file: Some(path),
            tiled_types_filter: TiledFilter::from(
                regex::RegexSet::new([
                    r"^properties_basic::.*",
                    r"^bevy_sprite::text2d::Text2d$",
                    r"^bevy_text::text::TextColor$",
                    r"^bevy_ecs::name::Name$",
                ])
                .unwrap(),
            ),
        }))
        // Examples helper plugins, such as the logic to pan and zoom the camera
        // This should not be used directly in your game (but you can always have a look)
        .add_plugins(helper::HelperPlugin)
        // We need to register all the custom types we want to use
        .register_type::<Biome>()
        .register_type::<SpawnPoint>()
        .register_type::<Resource>()
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands
        .spawn(TiledMap(
            asset_server.load("maps/hexagonal/finite_pointy_top_odd.tmx"),
        ))
        .observe(on_add_spawn)
        .observe(on_map_created);
}

// You just have to define your Components and make sure they are properly registered and reflected.
// They will be exported to the Tiled .json file so they can be imported then used from Tiled.
// Next time you load your map, they will be automatically added as components on tiles / objects / layers entities

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
struct Biome {
    ty: BiomeType,
    move_cost: usize,
    block_line_of_sight: bool,
}

#[derive(Default, Reflect, Debug)]
#[reflect(Default)]
enum BiomeType {
    #[default]
    Unknown,
    Plain,
    Desert,
    Forest,
    Mountain,
    Water,
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
enum SpawnPoint {
    #[default]
    Unknown,
    Player {
        color: Color,
        id: u32,
        other_obj: Option<Entity>,
    },
    Enemy(Color),
    Friendly,
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
enum Resource {
    #[default]
    Unknown,
    Wheat,
    Strawberry,
    Wood,
    Copper,
    Gold,
}

// This observer will be triggered every time a `SpawnType` component is added to an entity.
// We can use it to spawn additional entity / insert more components
fn on_add_spawn(
    add_spawn: On<Add, SpawnPoint>,
    spawn_query: Query<(&SpawnPoint, &GlobalTransform)>,
    mut _commands: Commands,
) {
    // Get the entity that triggered the observer
    let spawn_entity = add_spawn.event().entity;

    // Retrieve the entity components
    let Ok((spawn_type, global_transform)) = spawn_query.get(spawn_entity) else {
        return;
    };

    info!(
        "New SpawnPoint [{:?} @ {:?}]",
        spawn_type,
        global_transform.translation()
    );

    // Do some stuff based upon the spawn type value
    match spawn_type {
        SpawnPoint::Enemy(_) => {
            // Spawn another entity
            // _commands.spawn( ... );
        }
        SpawnPoint::Player { .. } => {
            // Add other components to the same entity
            // _commands.entity(spawn_entity).insert( ... );
        }
        _ => {}
    };
}

// This observer will be triggered after our map is loaded and all custom properties have been inserted
// We can use it to do some global initialization
fn on_map_created(
    map_created: On<TiledEvent<MapCreated>>,
    map_query: Query<&TiledMapStorage, With<TiledMap>>,
    tiles_query: Query<(&TilePos, Option<&Biome>, Option<&Resource>)>,
) {
    // Get the map entity and storage component
    let map_entity = map_created.event().origin;
    let Ok(map_storage) = map_query.get(map_entity) else {
        return;
    };

    // We will iterate over all tiles from our map and try to access our custom properties
    for ((_tile_id, _tileset_id), entities_list) in map_storage.tiles() {
        for tile_entity in entities_list {
            let Ok((pos, biome, resource)) = tiles_query.get(*tile_entity) else {
                continue;
            };

            // Here, we only print the content of our tile but we could also do some
            // global initialization.
            // A typical use case would be to initialize a resource so we can map a tile
            // position to a biome and / or a resource (which could be useful for pathfinding)

            if let Some(i) = biome {
                // Only print the first tile to avoid flooding the console
                info_once!("Found Biome [{:?} @ {:?}]", i, pos);
            }

            if let Some(i) = resource {
                // Only print the first tile to avoid flooding the console
                info_once!("Found Resource [{:?} @ {:?}]", i, pos);
            }
        }
    }
}
