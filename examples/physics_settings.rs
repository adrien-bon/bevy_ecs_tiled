// This example shows how to tune physics colliders generation, with the Avian or Rapier backend.
//
// Run it with `--features avian_debug` or `--features rapier_debug`.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

#[cfg(not(any(feature = "avian_debug", feature = "rapier_debug")))]
compile_error!("This example requires the `avian_debug` or `rapier_debug` feature");

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
        // bevy_ecs_tiled physics plugin, along with the physics engine:
        // see helper/physics.rs to know how a physics backend is selected
        .add_plugins(helper::physics::plugin)
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
                TiledPhysicsSettings::<helper::physics::Backend>::default(),
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
                TiledPhysicsSettings::<helper::physics::Backend>::default(),
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
                TiledPhysicsSettings::<helper::physics::Backend> {
                    // Only use the tiles collision shapes named 'collision'
                    tile_shapes_filter: TiledFilter::from(["collision"]),
                    ..default()
                },
            ))
            // Disable colliders for objects layers
            .observe(
                |layer_created: On<TiledEvent<LayerCreated>>,
                 layers_query: Query<&TiledLayer>,
                 mut commands: Commands| {
                    let layer = layer_created.origin;
                    if let Ok(TiledLayer::Objects) = layers_query.get(layer) {
                        commands
                            .entity(layer)
                            .insert(TiledColliderStrategy::Disabled);
                    }
                },
            );
        },
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/finite.tmx",
        "A finite orthogonal map with only object colliders",
        |c| {
            c.insert((
                TilemapAnchor::Center,
                // Disable colliders for the whole map...
                TiledPhysicsSettings::<helper::physics::Backend> {
                    collider_strategy: TiledColliderStrategy::Disabled,
                    ..default()
                },
            ))
            // ... but enable them for objects layers
            .observe(
                |layer_created: On<TiledEvent<LayerCreated>>,
                 layers_query: Query<&TiledLayer>,
                 mut commands: Commands| {
                    let layer = layer_created.origin;
                    if let Ok(TiledLayer::Objects) = layers_query.get(layer) {
                        commands
                            .entity(layer)
                            .insert(TiledColliderStrategy::Polyline);
                    }
                },
            );
        },
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/finite.tmx",
        "A finite orthogonal map with triangulated colliders, except for objects which use line strips",
        |c| {
            c.insert((
                TilemapAnchor::Center,
                // Strategy used to create colliders for the whole map...
                TiledPhysicsSettings::<helper::physics::Backend> {
                    collider_strategy: TiledColliderStrategy::Triangulation,
                    ..default()
                },
            ))
            // ... which can be overridden for a given layer (or a given object)
            // by inserting a `TiledColliderStrategy` component on it
            .observe(
                |layer_created: On<TiledEvent<LayerCreated>>,
                 layers_query: Query<&TiledLayer>,
                 mut commands: Commands| {
                    let layer = layer_created.origin;
                    if let Ok(TiledLayer::Objects) = layers_query.get(layer) {
                        commands
                            .entity(layer)
                            .insert(TiledColliderStrategy::LineStrip);
                    }
                },
            );
        },
    ));
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "maps/orthogonal/finite.tmx",
        "A finite orthogonal map with tiles colliders split in chunks of 4x4 tiles",
        |c| {
            c.insert((
                TilemapAnchor::Center,
                // Each chunk of tiles gets its own colliders: only the chunks whose tiles
                // are modified are computed again, instead of the whole layer.
                // This size can be overridden for a given layer by inserting
                // a `TiledColliderChunkSize` component on it.
                TiledPhysicsSettings::<helper::physics::Backend> {
                    tiles_chunk_size: TiledColliderChunkSize::Size(UVec2::splat(4)),
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
