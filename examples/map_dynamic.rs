//! This example shows how to modify a map after it has been spawned.
//!
//! Tiles, objects and images entities only hold Tiled data: rendering components
//! (tilemaps, sprites) and physics colliders are derived from these data and
//! are automatically updated when you modify them.
//!
//! Run with the `avian` feature to also display physics colliders.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

fn main() {
    let mut app = App::new();
    app
        // Bevy default plugins: prevent blur effect by changing default sampling
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done
        .add_plugins(TiledPlugin::default())
        // Examples helper plugins, such as the logic to pan and zoom the camera
        // This should not be used directly in your game (but you can always have a look)
        .add_plugins(helper::HelperPlugin)
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, modify_map);

    // Colliders are also updated when tiles or objects are modified
    #[cfg(feature = "avian")]
    app.add_plugins((
        TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default(),
        avian2d::prelude::PhysicsPlugins::default().with_length_unit(100.0),
        avian2d::prelude::PhysicsDebugPlugin,
    ));

    app.run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        TiledMap(asset_server.load("maps/orthogonal/finite.tmx")),
        TilemapAnchor::Center,
    ));

    commands.spawn((
        Text::new(
            "[1] Despawn a tile\n\
             [2] Change a tile\n\
             [3] Move a tile\n\
             [4] Spawn a new tile\n\
             [5] Despawn an object\n\
             [6] Change an object shape\n\
             [R] Reload the map",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

/// Pick a pseudo-random element from an iterator
fn pick<T>(seed: &mut u32, items: impl Iterator<Item = T>) -> Option<T> {
    let items = items.collect::<Vec<_>>();
    if items.is_empty() {
        return None;
    }
    // Simple linear congruential generator: good enough for this example
    *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    let index = (*seed >> 16) as usize % items.len();
    items.into_iter().nth(index)
}

#[allow(clippy::too_many_arguments)]
fn modify_map(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut seed: Local<u32>,
    maps_query: Query<(Entity, &TiledMap)>,
    layers_query: Query<(Entity, &TiledLayer)>,
    mut tiles_query: Query<(Entity, &mut TiledTileId, &mut TilePos), With<TiledTile>>,
    mut objects_query: Query<(Entity, &mut TiledObject)>,
    assets: Res<Assets<TiledMapAsset>>,
) {
    // Despawn a tile: it is removed from its tilemap
    if keyboard_input.just_pressed(KeyCode::Digit1)
        && let Some((entity, _, _)) = pick(&mut seed, tiles_query.iter())
    {
        info!("Despawn tile {entity}");
        commands.entity(entity).despawn();
    }

    // Change the tile ID of a tile: tilemap is updated with the new texture
    if keyboard_input.just_pressed(KeyCode::Digit2)
        && let Some((entity, mut tile_id, _)) = pick(&mut seed, tiles_query.iter_mut())
    {
        // Our tileset contains 7 tiles
        tile_id.0 = (tile_id.0 + 1) % 7;
        info!("Change tile {entity} to {}", tile_id.0);
    }

    // Move a tile one step to the right, if it stays within the map
    if keyboard_input.just_pressed(KeyCode::Digit3)
        && let Some((_, map)) = maps_query.iter().next()
        && let Some(map_asset) = assets.get(&map.0)
        && let Some((entity, _, mut tile_pos)) = pick(
            &mut seed,
            tiles_query
                .iter_mut()
                .filter(|(_, _, pos)| pos.x + 1 < map_asset.tilemap_size.x),
        )
    {
        tile_pos.x += 1;
        info!("Move tile {entity} to {:?}", *tile_pos);
    }

    // Spawn a new tile: only data components are needed,
    // a tilemap will be created for this tile tileset if needed
    if keyboard_input.just_pressed(KeyCode::Digit4)
        && let Some((_, map)) = maps_query.iter().next()
        && let Some(map_asset) = assets.get(&map.0)
        && let Some((layer, _)) = pick(
            &mut seed,
            layers_query
                .iter()
                .filter(|(_, layer)| matches!(layer, TiledLayer::Tiles)),
        )
    {
        let size = map_asset.tilemap_size;
        let position = pick(&mut seed, 0..size.x * size.y).unwrap_or_default();
        let tile_pos = TilePos::new(position % size.x, position / size.x);
        let tile_id = pick(&mut seed, 0..7).unwrap_or_default();
        info!("Spawn tile {tile_id} at {tile_pos:?}");
        commands.spawn((
            Name::new("Manually spawned tile"),
            TiledTile,
            ChildOf(layer),
            TiledTilesetId(0),
            TiledTileId(tile_id),
            tile_pos,
        ));
    }

    // Despawn an object: its visual and colliders are despawned as well
    if keyboard_input.just_pressed(KeyCode::Digit5)
        && let Some((entity, _)) = pick(&mut seed, objects_query.iter())
    {
        info!("Despawn object {entity}");
        commands.entity(entity).despawn();
    }

    // Change an object shape: its colliders are spawned again
    if keyboard_input.just_pressed(KeyCode::Digit6)
        && let Some((entity, mut object)) = pick(
            &mut seed,
            objects_query.iter_mut().filter(|(_, object)| {
                matches!(
                    **object,
                    TiledObject::Ellipse { .. } | TiledObject::Rectangle { .. }
                )
            }),
        )
    {
        *object = match *object {
            TiledObject::Ellipse { width, height } => TiledObject::Rectangle { width, height },
            TiledObject::Rectangle { width, height } => TiledObject::Ellipse { width, height },
            ref other => other.clone(),
        };
        info!("Change object {entity} to {}", object.kind());
    }

    // Respawn the map from its asset: all modifications are lost
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for (entity, _) in maps_query.iter() {
            commands.entity(entity).insert(RespawnTiledMap);
        }
    }
}
