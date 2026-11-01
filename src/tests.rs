//! Headless tests checking that rendering and physics components follow
//! Tiled data components when they are modified after the map has been spawned.

use crate::prelude::*;
use bevy::{asset::LoadState, prelude::*};
use std::time::{Duration, Instant};

const DYNAMIC_MAP: &str = "tests/dynamic/dynamic_map.tmx";

/// Counters updated by observers
#[derive(Resource, Default)]
struct Received {
    tilemaps_created: Vec<Entity>,
    /// Layers, tiles and objects found in map storage when receiving [`MapCreated`]
    storage_on_map_created: Option<(usize, usize, usize)>,
}

/// Build an [`App`] with all the plugin internals but without bevy_ecs_tilemap
/// [`TilemapPlugin`](bevy_ecs_tilemap::TilemapPlugin), which requires a renderer.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        bevy::image::ImagePlugin::default(),
        bevy::image::TextureAtlasPlugin,
    ));
    // Image loader is usually registered by the render plugin
    app.register_asset_loader(bevy::image::ImageLoader::new(
        bevy::image::CompressedImageFormats::empty(),
    ));
    app.insert_resource(TiledPluginConfig {
        tiled_types_export_file: None,
        tiled_types_filter: TiledFilter::None,
    });
    app.insert_resource(crate::tiled::cache::TiledResourceCache::new());
    app.add_plugins((
        crate::tiled::map::plugin,
        crate::tiled::animation::plugin,
        crate::tiled::cache::plugin,
        crate::tiled::event::plugin,
        crate::tiled::image::plugin,
        crate::tiled::layer::plugin,
        crate::tiled::object::plugin,
        crate::tiled::tile::plugin,
        crate::tiled::sets::plugin,
        crate::tiled::filter::plugin,
    ));
    app.init_resource::<Received>();
    app.add_observer(
        |ev: On<TiledEvent<TilemapCreated>>, mut received: ResMut<Received>| {
            // Events are propagated to parent entities: only count the original one
            let ev = ev.event();
            if ev.entity == ev.origin {
                received.tilemaps_created.push(ev.origin);
            }
        },
    );
    app.add_observer(
        |ev: On<TiledEvent<MapCreated>>,
         mut received: ResMut<Received>,
         storage_query: Query<&TiledMapStorage>| {
            let storage = storage_query.get(ev.event().origin).unwrap();
            received.storage_on_map_created = Some((
                storage.layers().count(),
                storage.tiles().map(|(_, e)| e.len()).sum(),
                storage.objects().count(),
            ));
        },
    );
    app
}

/// Spawn the test map and run the app until it is spawned
fn spawn_test_map(app: &mut App) -> Entity {
    spawn_map(app, DYNAMIC_MAP)
}

/// Spawn the given map and run the app until it is spawned
fn spawn_map(app: &mut App, path: &str) -> Entity {
    let handle = app
        .world()
        .resource::<AssetServer>()
        .load::<TiledMapAsset>(path.to_owned());
    let map_entity = app.world_mut().spawn(TiledMap(handle.clone())).id();

    let deadline = Instant::now() + Duration::from_secs(10);
    while app
        .world()
        .resource::<Received>()
        .storage_on_map_created
        .is_none()
    {
        assert!(Instant::now() < deadline, "test map did not spawn in time");
        if let LoadState::Failed(error) = app.world().resource::<AssetServer>().load_state(&handle)
        {
            panic!("test map failed to load: {error}");
        }
        app.update();
        std::thread::sleep(Duration::from_millis(1));
    }
    // Make sure PostUpdate systems ran after the map has been spawned
    app.update();
    map_entity
}

fn layer_by_name(app: &mut App, name: &str) -> Entity {
    app.world_mut()
        .query_filtered::<(Entity, &TiledName), With<TiledLayer>>()
        .iter(app.world())
        .find(|(_, n)| n.0 == name)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("missing layer {name}"))
}

fn tile_at(app: &mut App, layer: Entity, pos: TilePos) -> Entity {
    app.world_mut()
        .query_filtered::<(Entity, &TilePos, &ChildOf), With<TiledTile>>()
        .iter(app.world())
        .find(|(_, p, c)| **p == pos && c.parent() == layer)
        .map(|(e, _, _)| e)
        .unwrap_or_else(|| panic!("missing tile at {pos:?}"))
}

#[cfg(feature = "render")]
/// Returns the tilemap of a layer for a given tileset ID
fn tilemap_of(app: &App, layer: Entity, tileset_id: u32) -> Option<Entity> {
    let world = app.world();
    world
        .get::<TiledTilemaps>(layer)?
        .iter()
        .find(|e| world.get::<TiledTilesetId>(*e).map(|id| id.0) == Some(tileset_id))
}

#[cfg(feature = "render")]
fn tilemaps_count(app: &App, layer: Entity) -> usize {
    app.world()
        .get::<TiledTilemaps>(layer)
        .map(|t| t.len())
        .unwrap_or(0)
}

#[cfg(feature = "render")]
fn storage_at(app: &App, tilemap: Entity, pos: TilePos) -> Option<Entity> {
    app.world()
        .get::<TileStorage>(tilemap)
        .and_then(|s| s.checked_get(&pos))
}

fn object_by_name(app: &mut App, name: &str) -> Entity {
    app.world_mut()
        .query_filtered::<(Entity, &TiledName), With<TiledObject>>()
        .iter(app.world())
        .find(|(_, n)| n.0 == name)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("missing object {name}"))
}

#[test]
#[cfg(feature = "render")]
fn map_load_spawns_data_and_render_components() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);

    // Created events are only sent once all the map data have been spawned
    assert_eq!(
        app.world().resource::<Received>().storage_on_map_created,
        Some((3, 4, 3))
    );

    let ground = layer_by_name(&mut app, "ground");
    let decoration = layer_by_name(&mut app, "decoration");

    // Tiles and tilemaps are both direct children of the layer
    let tile = tile_at(&mut app, ground, TilePos::new(0, 3));
    let tilemap = tilemap_of(&app, ground, 0).expect("missing tilemap");
    assert_eq!(
        app.world().get::<ChildOf>(tilemap).unwrap().parent(),
        ground
    );
    assert_eq!(
        app.world().get::<TilemapId>(tile),
        Some(&TilemapId(tilemap))
    );
    assert_eq!(storage_at(&app, tilemap, TilePos::new(0, 3)), Some(tile));
    assert_eq!(app.world().get::<TiledMapReference>(tile).unwrap().0, map);

    // One tilemap per layer and tileset
    assert_eq!(tilemaps_count(&app, ground), 2);
    assert_eq!(tilemaps_count(&app, decoration), 1);
    assert_eq!(app.world().resource::<Received>().tilemaps_created.len(), 3);

    // Tile objects have a visual
    let tile_object = object_by_name(&mut app, "tile_object");
    let visuals = app.world().get::<TiledObjectVisuals>(tile_object).unwrap();
    assert_eq!(visuals.len(), 1);
    assert!(app.world().get::<Sprite>(visuals[0]).is_some());
    let rectangle = object_by_name(&mut app, "rectangle");
    assert!(app.world().get::<TiledObjectVisuals>(rectangle).is_none());
}

#[test]
#[cfg(feature = "render")]
fn despawning_tiles_updates_tilemaps_and_storage() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");

    // The only tile from tileset 1 in the ground layer
    let tile = tile_at(&mut app, ground, TilePos::new(3, 0));
    let tilemap = tilemap_of(&app, ground, 1).expect("missing tilemap");
    app.world_mut().entity_mut(tile).despawn();
    app.update();

    // Tilemap does not contain any tile anymore: it has been despawned
    assert!(app.world().get_entity(tilemap).is_err());
    assert_eq!(tilemaps_count(&app, ground), 1);
    let storage = app.world().get::<TiledMapStorage>(map).unwrap();
    assert!(storage.tile_entities(1, 1).is_empty());
    assert_eq!(storage.tile_id(tile), None);

    // Despawn a tile which shares its tilemap with another one
    let tile = tile_at(&mut app, ground, TilePos::new(0, 3));
    let tilemap = tilemap_of(&app, ground, 0).expect("missing tilemap");
    app.world_mut().entity_mut(tile).despawn();
    app.update();
    assert_eq!(storage_at(&app, tilemap, TilePos::new(0, 3)), None);
    assert!(storage_at(&app, tilemap, TilePos::new(1, 3)).is_some());
    let storage = app.world().get::<TiledMapStorage>(map).unwrap();
    assert_eq!(storage.tile_entities(0, 0).len(), 1);
}

#[test]
#[cfg(feature = "render")]
fn modifying_tiles_updates_tilemaps_and_storage() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    let tilemap_0 = tilemap_of(&app, ground, 0).unwrap();
    let tilemap_1 = tilemap_of(&app, ground, 1).unwrap();

    // Move a tile
    let tile = tile_at(&mut app, ground, TilePos::new(0, 3));
    *app.world_mut().get_mut::<TilePos>(tile).unwrap() = TilePos::new(2, 2);
    app.update();
    assert_eq!(storage_at(&app, tilemap_0, TilePos::new(0, 3)), None);
    assert_eq!(storage_at(&app, tilemap_0, TilePos::new(2, 2)), Some(tile));

    // Change its tile ID
    app.world_mut().get_mut::<TiledTileId>(tile).unwrap().0 = 3;
    app.update();
    assert_eq!(
        app.world().get::<TileTextureIndex>(tile),
        Some(&TileTextureIndex(3))
    );
    let storage = app.world().get::<TiledMapStorage>(map).unwrap();
    assert_eq!(storage.tile_id(tile), Some((0, 3)));

    // Change its tileset: it moves to the other tilemap of the layer
    app.world_mut().get_mut::<TiledTilesetId>(tile).unwrap().0 = 1;
    app.update();
    assert_eq!(
        app.world().get::<TilemapId>(tile),
        Some(&TilemapId(tilemap_1))
    );
    assert_eq!(storage_at(&app, tilemap_0, TilePos::new(2, 2)), None);
    assert_eq!(storage_at(&app, tilemap_1, TilePos::new(2, 2)), Some(tile));
    let storage = app.world().get::<TiledMapStorage>(map).unwrap();
    assert_eq!(storage.tile_id(tile), Some((1, 3)));
}

#[test]
#[cfg(feature = "render")]
fn spawning_tiles_creates_tilemaps() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);
    let decoration = layer_by_name(&mut app, "decoration");
    assert!(tilemap_of(&app, decoration, 1).is_none());

    // Only the data components are needed to spawn a new tile
    let tile = app
        .world_mut()
        .spawn((
            TiledTile,
            ChildOf(decoration),
            TiledTilesetId(1),
            TiledTileId(2),
            TilePos::new(2, 2),
        ))
        .id();
    app.update();

    let tilemap = tilemap_of(&app, decoration, 1).expect("tilemap was not created");
    assert_eq!(storage_at(&app, tilemap, TilePos::new(2, 2)), Some(tile));
    assert_eq!(
        app.world().resource::<Received>().tilemaps_created.last(),
        Some(&tilemap)
    );
    assert_eq!(app.world().get::<TiledMapReference>(tile).unwrap().0, map);
    let storage = app.world().get::<TiledMapStorage>(map).unwrap();
    assert_eq!(storage.tile_entities(1, 2), &[tile]);
}

#[test]
fn modifying_objects_updates_visuals() {
    let mut app = test_app();
    spawn_test_map(&mut app);
    let tile_object = object_by_name(&mut app, "tile_object");
    let visual = app.world().get::<TiledObjectVisuals>(tile_object).unwrap()[0];

    // Change the object tile: its visual is spawned again
    if let TiledObject::Tile { tile_id, .. } =
        &mut *app.world_mut().get_mut::<TiledObject>(tile_object).unwrap()
    {
        *tile_id = 2;
    }
    app.update();
    assert!(app.world().get_entity(visual).is_err());
    let visuals = app.world().get::<TiledObjectVisuals>(tile_object).unwrap();
    assert_eq!(visuals.len(), 1);

    // Turn it into a rectangle: it does not have any visual anymore
    *app.world_mut().get_mut::<TiledObject>(tile_object).unwrap() = TiledObject::Rectangle {
        width: 10.,
        height: 10.,
    };
    app.update();
    assert!(app.world().get::<TiledObjectVisuals>(tile_object).is_none());

    // Remove the TiledObject component of another tile object: its visual is despawned
    let tile_object = object_by_name(&mut app, "tile_object_with_override");
    let visual = app.world().get::<TiledObjectVisuals>(tile_object).unwrap()[0];
    app.world_mut()
        .entity_mut(tile_object)
        .remove::<TiledObject>();
    app.update();
    assert!(app.world().get_entity(visual).is_err());
    assert!(app.world().get_entity(tile_object).is_ok());
    assert!(app.world().get::<TiledObjectVisuals>(tile_object).is_none());
}

/// Tiles of a layer, as registered in its index
fn layer_index(app: &App, layer: Entity) -> Vec<(TilePos, Entity)> {
    let mut tiles = app
        .world()
        .get::<crate::tiled::tile::TiledLayerTiles>(layer)
        .unwrap()
        .tiles()
        .iter()
        .map(|(pos, e)| (*pos, *e))
        .collect::<Vec<_>>();
    tiles.sort();
    tiles
}

/// Tiles of a layer, from their actual components
fn layer_tiles(app: &mut App, layer: Entity) -> Vec<(TilePos, Entity)> {
    let mut tiles = app
        .world_mut()
        .query_filtered::<(Entity, &TilePos, &ChildOf), With<TiledTile>>()
        .iter(app.world())
        .filter(|(_, _, c)| c.parent() == layer)
        .map(|(e, pos, _)| (*pos, e))
        .collect::<Vec<_>>();
    tiles.sort();
    tiles
}

#[test]
fn moving_a_tile_onto_another_replaces_it() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    let a = tile_at(&mut app, ground, TilePos::new(0, 3));
    let b = tile_at(&mut app, ground, TilePos::new(1, 3));

    // Move A onto B: B is despawned
    *app.world_mut().get_mut::<TilePos>(a).unwrap() = TilePos::new(1, 3);
    app.update();
    assert!(app.world().get_entity(b).is_err());
    let storage = app.world().get::<TiledMapStorage>(map).unwrap();
    assert_eq!(storage.tile_id(b), None);
    assert_eq!(layer_index(&app, ground), layer_tiles(&mut app, ground));

    #[cfg(feature = "render")]
    assert_tilemaps_count_match_storage(&mut app);

    // Move A back: its previous position is empty
    *app.world_mut().get_mut::<TilePos>(a).unwrap() = TilePos::new(0, 3);
    app.update();
    assert!(
        layer_tiles(&mut app, ground)
            .iter()
            .all(|(pos, _)| *pos != TilePos::new(1, 3))
    );
    assert_eq!(layer_index(&app, ground), layer_tiles(&mut app, ground));

    #[cfg(feature = "render")]
    {
        let tilemap = tilemap_of(&app, ground, 0).unwrap();
        assert_eq!(storage_at(&app, tilemap, TilePos::new(1, 3)), None);
        assert_eq!(storage_at(&app, tilemap, TilePos::new(0, 3)), Some(a));
    }
}

#[test]
fn swapping_tiles_keeps_both() {
    let mut app = test_app();
    spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    let a = tile_at(&mut app, ground, TilePos::new(0, 3));
    let b = tile_at(&mut app, ground, TilePos::new(1, 3));

    *app.world_mut().get_mut::<TilePos>(a).unwrap() = TilePos::new(1, 3);
    *app.world_mut().get_mut::<TilePos>(b).unwrap() = TilePos::new(0, 3);
    app.update();
    assert_eq!(tile_at(&mut app, ground, TilePos::new(1, 3)), a);
    assert_eq!(tile_at(&mut app, ground, TilePos::new(0, 3)), b);
    assert_eq!(layer_index(&app, ground), layer_tiles(&mut app, ground));

    // Index is still valid: moving A to an empty position does not despawn anything
    *app.world_mut().get_mut::<TilePos>(a).unwrap() = TilePos::new(2, 2);
    app.update();
    assert!(app.world().get_entity(b).is_ok());
    assert_eq!(layer_index(&app, ground), layer_tiles(&mut app, ground));
}

#[test]
fn spawning_a_tile_onto_another_replaces_it() {
    let mut app = test_app();
    spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    let a = tile_at(&mut app, ground, TilePos::new(0, 3));

    // Another tileset: tiles are still replaced
    let new_tile = app
        .world_mut()
        .spawn((
            TiledTile,
            ChildOf(ground),
            TiledTilesetId(1),
            TiledTileId(2),
            TilePos::new(0, 3),
        ))
        .id();
    app.update();
    assert!(app.world().get_entity(a).is_err());
    assert_eq!(tile_at(&mut app, ground, TilePos::new(0, 3)), new_tile);
    assert_eq!(layer_index(&app, ground), layer_tiles(&mut app, ground));
}

#[test]
fn layer_index_follows_tiles() {
    let mut app = test_app();
    spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    let decoration = layer_by_name(&mut app, "decoration");
    assert_eq!(layer_index(&app, ground), layer_tiles(&mut app, ground));
    assert_eq!(
        layer_index(&app, decoration),
        layer_tiles(&mut app, decoration)
    );
    assert_eq!(layer_index(&app, ground).len(), 3);

    // Add a tile
    let tile = app
        .world_mut()
        .spawn((
            TiledTile,
            ChildOf(ground),
            TiledTilesetId(0),
            TiledTileId(1),
            TilePos::new(2, 2),
        ))
        .id();
    app.update();
    assert!(layer_index(&app, ground).contains(&(TilePos::new(2, 2), tile)));

    // Move it: previous position is no longer in the index
    *app.world_mut().get_mut::<TilePos>(tile).unwrap() = TilePos::new(2, 1);
    app.update();
    assert_eq!(layer_index(&app, ground), layer_tiles(&mut app, ground));
    assert!(
        layer_index(&app, ground)
            .iter()
            .all(|(pos, _)| *pos != TilePos::new(2, 2))
    );

    // Despawn it: it is removed from the index
    app.world_mut().entity_mut(tile).despawn();
    app.update();
    assert_eq!(layer_index(&app, ground).len(), 3);
    assert_eq!(layer_index(&app, ground), layer_tiles(&mut app, ground));
}

#[test]
#[cfg(feature = "render")]
fn tiles_outside_of_map_are_not_rendered() {
    use crate::tiled::tile::render::TiledTileRenderState;

    let mut app = test_app();
    spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    let tilemap = tilemap_of(&app, ground, 0).unwrap();
    let tile = tile_at(&mut app, ground, TilePos::new(0, 3));

    // Move the tile outside of the map: it is detached from its tilemap
    *app.world_mut().get_mut::<TilePos>(tile).unwrap() = TilePos::new(10, 10);
    app.update();
    assert!(app.world().get::<TilemapId>(tile).is_none());
    assert_eq!(storage_at(&app, tilemap, TilePos::new(0, 3)), None);
    let state = *app.world().get::<TiledTileRenderState>(tile).unwrap();
    assert_eq!(state.tilemap, None);
    assert_eq!(state.position, TilePos::new(10, 10));

    // The tile is not processed again on next frame
    app.update();
    let state = app
        .world_mut()
        .query::<Ref<TiledTileRenderState>>()
        .get(app.world(), tile)
        .unwrap();
    assert!(!state.is_changed());

    // Move it back inside the map: it is rendered again
    *app.world_mut().get_mut::<TilePos>(tile).unwrap() = TilePos::new(2, 2);
    app.update();
    assert_eq!(
        app.world().get::<TilemapId>(tile),
        Some(&TilemapId(tilemap))
    );
    assert_eq!(storage_at(&app, tilemap, TilePos::new(2, 2)), Some(tile));
}

#[test]
fn spawned_objects_inherit_map_reference() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);
    let objects = layer_by_name(&mut app, "objects");

    // Only the TiledObject and its parent layer are needed
    let object = app
        .world_mut()
        .spawn((
            TiledObject::Tile {
                width: 16.,
                height: 16.,
                tileset_id: 0,
                tile_id: 1,
                flip_x: false,
                flip_y: false,
            },
            TiledObjectId(99),
            ChildOf(objects),
        ))
        .id();
    app.update();

    assert_eq!(app.world().get::<TiledMapReference>(object).unwrap().0, map);
    let storage = app.world().get::<TiledMapStorage>(map).unwrap();
    assert_eq!(storage.object_entity(99), Some(object));
    assert_eq!(
        app.world()
            .get::<TiledObjectVisuals>(object)
            .map(|v| v.len()),
        Some(1)
    );
}

/// Check that the tile count of each tilemap matches the content of its storage
#[cfg(feature = "render")]
fn assert_tilemaps_count_match_storage(app: &mut App) {
    use crate::tiled::tile::render::TiledTilemapTileCount;

    for (entity, storage, count) in app
        .world_mut()
        .query::<(Entity, &TileStorage, &TiledTilemapTileCount)>()
        .iter(app.world())
    {
        assert_eq!(
            count.0 as usize,
            storage.iter().flatten().count(),
            "wrong tile count for tilemap {entity}"
        );
    }
}

/// Check that the map storage matches the actual tiles, and that each tile knows its index in it
fn assert_storage_matches_tiles(app: &mut App, map: Entity) {
    use crate::tiled::map::storage::TiledTileStorageIndex;

    let tiles = app
        .world_mut()
        .query_filtered::<(
            Entity,
            &TiledTilesetId,
            &TiledTileId,
            &TiledTileStorageIndex,
        ), With<TiledTile>>()
        .iter(app.world())
        .map(|(e, tileset_id, tile_id, index)| (e, (tileset_id.0, tile_id.0), index.0))
        .collect::<Vec<_>>();
    let storage = app.world().get::<TiledMapStorage>(map).unwrap();
    assert_eq!(
        storage.tiles().map(|(_, e)| e.len()).sum::<usize>(),
        tiles.len()
    );
    for (entity, key, index) in tiles {
        let entities = storage.tile_entities(key.0, key.1);
        let position = entities.iter().position(|e| *e == entity);
        assert!(position.is_some(), "tile {entity} is not in storage");
        assert_eq!(index, Some((key, position.unwrap())));
    }

    #[cfg(feature = "render")]
    assert_tilemaps_count_match_storage(app);
}

#[test]
fn storage_follows_tiles() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    assert_storage_matches_tiles(&mut app, map);

    // Several tiles sharing the same tileset ID + tile ID
    let tiles = (0..6)
        .map(|i| {
            app.world_mut()
                .spawn((
                    TiledTile,
                    ChildOf(ground),
                    TiledTilesetId(0),
                    TiledTileId(0),
                    TilePos::new(i % 4, 1 + i / 4),
                ))
                .id()
        })
        .collect::<Vec<_>>();
    app.update();
    assert_storage_matches_tiles(&mut app, map);

    // Despawn some of them and change the tile ID of others
    app.world_mut().entity_mut(tiles[0]).despawn();
    app.world_mut().entity_mut(tiles[3]).despawn();
    app.world_mut().get_mut::<TiledTileId>(tiles[1]).unwrap().0 = 2;
    app.world_mut()
        .get_mut::<TiledTilesetId>(tiles[5])
        .unwrap()
        .0 = 1;
    app.update();
    assert_storage_matches_tiles(&mut app, map);

    app.world_mut().entity_mut(tiles[2]).despawn();
    app.world_mut().get_mut::<TiledTileId>(tiles[1]).unwrap().0 = 0;
    app.update();
    assert_storage_matches_tiles(&mut app, map);
}

/// A map entity whose asset is not available yet, with a single tile
#[cfg(any(feature = "render", feature = "physics"))]
struct MapWithoutAsset {
    /// Handle to the loaded test map asset
    loaded: Handle<TiledMapAsset>,
    /// Handle used by the map entity, without any asset yet
    reserved: Handle<TiledMapAsset>,
    layer: Entity,
    #[cfg_attr(not(feature = "render"), allow(dead_code))]
    tile: Entity,
}

#[cfg(any(feature = "render", feature = "physics"))]
impl MapWithoutAsset {
    fn spawn(app: &mut App) -> Self {
        let loaded = app
            .world()
            .resource::<AssetServer>()
            .load::<TiledMapAsset>(DYNAMIC_MAP);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !app
            .world()
            .resource::<AssetServer>()
            .is_loaded_with_dependencies(&loaded)
        {
            assert!(Instant::now() < deadline, "test map did not load in time");
            app.update();
            std::thread::sleep(Duration::from_millis(1));
        }

        let reserved = app
            .world()
            .resource::<Assets<TiledMapAsset>>()
            .reserve_handle();
        let map = app.world_mut().spawn(TiledMap(reserved.clone())).id();
        let layer = app
            .world_mut()
            .spawn((TiledLayer::Tiles, TiledMapReference(map), ChildOf(map)))
            .id();
        let tile = app
            .world_mut()
            .spawn((
                TiledTile,
                ChildOf(layer),
                TiledTilesetId(0),
                TiledTileId(0),
                TilePos::new(1, 1),
            ))
            .id();
        Self {
            loaded,
            reserved,
            layer,
            tile,
        }
    }

    /// Make the asset available for the map entity
    fn insert_asset(&self, app: &mut App) {
        let mut assets = app.world_mut().resource_mut::<Assets<TiledMapAsset>>();
        let asset = assets.remove(self.loaded.id()).unwrap();
        let _ = assets.insert(self.reserved.id(), asset);
    }
}

#[test]
#[cfg(feature = "render")]
fn tiles_are_rendered_once_map_asset_is_available() {
    let mut app = test_app();
    let map = MapWithoutAsset::spawn(&mut app);
    for _ in 0..3 {
        app.update();
    }
    assert!(app.world().get::<TilemapId>(map.tile).is_none());

    map.insert_asset(&mut app);
    app.update();
    let tilemap = app
        .world()
        .get::<TilemapId>(map.tile)
        .expect("tile is not rendered")
        .0;
    assert_eq!(
        storage_at(&app, tilemap, TilePos::new(1, 1)),
        Some(map.tile)
    );
    assert_eq!(
        app.world().get::<ChildOf>(tilemap).unwrap().parent(),
        map.layer
    );
}

#[test]
fn repeated_images_are_sized_on_first_frame() {
    let mut app = test_app();
    spawn_test_map(&mut app);
    let layer = layer_by_name(&mut app, "objects");

    // A camera seeing a 400 pixels wide area
    app.world_mut().spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            area: Rect::new(-200., -100., 200., 100.),
            ..OrthographicProjection::default_2d()
        }),
    ));

    let image = app
        .world_mut()
        .spawn((
            TiledImage {
                base_size: Vec2::new(16., 16.),
                repeat_x: true,
                ..default()
            },
            ChildOf(layer),
        ))
        .id();
    app.update();

    // On the very first frame, the image already covers the camera area along the X axis
    let sprite = app.world().get::<Sprite>(image).unwrap();
    let size = sprite.custom_size.unwrap();
    assert!(size.x >= 400., "repeated image width is {}", size.x);
    assert_eq!(size.y, 16.);
}

#[cfg(feature = "physics")]
mod physics {
    use super::*;
    use ::geo::{Area, BoundingRect};

    /// Physics backend which only spawns an entity holding the collider polygons
    #[derive(Default, Clone, Debug, Reflect)]
    pub(super) struct TestBackend;

    impl TiledPhysicsBackend for TestBackend {
        fn spawn_colliders(
            &self,
            commands: &mut Commands,
            _source: TiledColliderSource,
            origin: Entity,
            _multi_polygons_list: Vec<geo::MultiPolygon<f32>>,
        ) -> Option<Entity> {
            Some(commands.spawn(ChildOf(origin)).id())
        }
    }

    fn colliders_area(app: &App, entity: Entity) -> Vec<f32> {
        app.world()
            .get::<TiledColliders>(entity)
            .into_iter()
            .flat_map(|c| c.iter())
            .map(|e| {
                app.world()
                    .get::<TiledColliderPolygons>(e)
                    .unwrap()
                    .iter()
                    .map(|p| p.unsigned_area())
                    .sum()
            })
            .collect()
    }

    /// Bounding box size and center of an entity colliders, relative to `origin`
    fn colliders_bounds(app: &App, entity: Entity, origin: Vec2) -> (Vec2, Vec2) {
        let rect = app
            .world()
            .get::<TiledColliders>(entity)
            .into_iter()
            .flat_map(|c| c.iter())
            .flat_map(|e| {
                app.world()
                    .get::<TiledColliderPolygons>(e)
                    .unwrap()
                    .0
                    .clone()
            })
            .filter_map(|p| p.bounding_rect())
            .reduce(|a, b| {
                ::geo::Rect::new(
                    ::geo::Coord {
                        x: a.min().x.min(b.min().x),
                        y: a.min().y.min(b.min().y),
                    },
                    ::geo::Coord {
                        x: a.max().x.max(b.max().x),
                        y: a.max().y.max(b.max().y),
                    },
                )
            })
            .expect("no collider");
        let center = rect.center();
        (
            Vec2::new(rect.width(), rect.height()),
            Vec2::new(center.x, center.y) - origin,
        )
    }

    #[test]
    fn colliders_follow_tiles_flips() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        let map = spawn_test_map(&mut app);
        let decoration = layer_by_name(&mut app, "decoration");
        assert!(app.world().get::<TiledColliders>(decoration).is_none());

        // Tile 3 has a collider on its left half
        let tile_pos = TilePos::new(2, 1);
        let tile = app
            .world_mut()
            .spawn((
                TiledTile,
                ChildOf(decoration),
                TiledTilesetId(0),
                TiledTileId(3),
                tile_pos,
            ))
            .id();
        app.update();

        let tile_center = {
            let handle = &app.world().get::<TiledMap>(map).unwrap().0;
            let assets = app.world().resource::<Assets<TiledMapAsset>>();
            assets.get(handle).unwrap().tile_relative_position(
                &tile_pos,
                &TilemapTileSize::new(16., 16.),
                &TilemapAnchor::default(),
            )
        };
        assert_eq!(
            colliders_bounds(&app, decoration, tile_center),
            (Vec2::new(8., 16.), Vec2::new(-4., 0.))
        );

        // Horizontal flip: collider is on the right half
        *app.world_mut().get_mut::<TileFlip>(tile).unwrap() = TileFlip {
            x: true,
            ..default()
        };
        app.update();
        assert_eq!(
            colliders_bounds(&app, decoration, tile_center),
            (Vec2::new(8., 16.), Vec2::new(4., 0.))
        );

        // Diagonal flip: collider is on the top half
        *app.world_mut().get_mut::<TileFlip>(tile).unwrap() = TileFlip {
            d: true,
            ..default()
        };
        app.update();
        assert_eq!(
            colliders_bounds(&app, decoration, tile_center),
            (Vec2::new(16., 8.), Vec2::new(0., 4.))
        );

        // Diagonal and vertical flips: collider is on the bottom half
        *app.world_mut().get_mut::<TileFlip>(tile).unwrap() = TileFlip {
            y: true,
            d: true,
            ..default()
        };
        app.update();
        assert_eq!(
            colliders_bounds(&app, decoration, tile_center),
            (Vec2::new(16., 8.), Vec2::new(0., -4.))
        );
    }

    #[test]
    fn replaced_tiles_colliders_are_removed() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        spawn_test_map(&mut app);
        let ground = layer_by_name(&mut app, "ground");
        assert_eq!(colliders_area(&app, ground), vec![512.]);

        // Move a tile onto the other one, then move it back
        let tile = tile_at(&mut app, ground, TilePos::new(0, 3));
        *app.world_mut().get_mut::<TilePos>(tile).unwrap() = TilePos::new(1, 3);
        app.update();
        *app.world_mut().get_mut::<TilePos>(tile).unwrap() = TilePos::new(0, 3);
        app.update();

        // The replaced tile does not have a collider anymore
        assert_eq!(colliders_area(&app, ground), vec![256.]);
    }

    #[test]
    fn tiles_outside_of_map_have_no_collider() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        spawn_test_map(&mut app);
        let ground = layer_by_name(&mut app, "ground");
        assert_eq!(colliders_area(&app, ground), vec![512.]);

        let tile = tile_at(&mut app, ground, TilePos::new(0, 3));
        *app.world_mut().get_mut::<TilePos>(tile).unwrap() = TilePos::new(10, 10);
        app.update();
        assert_eq!(colliders_area(&app, ground), vec![256.]);

        *app.world_mut().get_mut::<TilePos>(tile).unwrap() = TilePos::new(0, 3);
        app.update();
        assert_eq!(colliders_area(&app, ground), vec![512.]);
    }

    #[test]
    fn spawned_objects_get_colliders() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        spawn_test_map(&mut app);
        let objects = layer_by_name(&mut app, "objects");

        // No TiledMapReference: it is inherited from the parent layer
        let object = app
            .world_mut()
            .spawn((
                TiledObject::Rectangle {
                    width: 4.,
                    height: 5.,
                },
                TiledObjectId(99),
                ChildOf(objects),
            ))
            .id();
        app.update();
        assert_eq!(colliders_area(&app, object), vec![20.]);
    }

    #[test]
    fn colliders_are_spawned_once_map_asset_is_available() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        let map = MapWithoutAsset::spawn(&mut app);
        for _ in 0..3 {
            app.update();
        }
        assert!(colliders_area(&app, map.layer).is_empty());

        map.insert_asset(&mut app);
        app.update();
        assert_eq!(colliders_area(&app, map.layer), vec![256.]);
    }

    #[test]
    fn colliders_follow_tiles_and_objects() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        spawn_test_map(&mut app);
        let ground = layer_by_name(&mut app, "ground");

        // Two adjacent 16x16 tiles with a full collider, merged in a single collider
        assert_eq!(colliders_area(&app, ground), vec![512.]);

        // Despawn one of them
        let tile = tile_at(&mut app, ground, TilePos::new(0, 3));
        app.world_mut().entity_mut(tile).despawn();
        app.update();
        assert_eq!(colliders_area(&app, ground), vec![256.]);

        // Change the other one to a tile without collision
        let tile = tile_at(&mut app, ground, TilePos::new(1, 3));
        app.world_mut().get_mut::<TiledTilesetId>(tile).unwrap().0 = 1;
        app.update();
        assert!(colliders_area(&app, ground).is_empty());

        // Objects colliders follow the object shape
        let rectangle = object_by_name(&mut app, "rectangle");
        assert_eq!(colliders_area(&app, rectangle), vec![128.]);
        *app.world_mut().get_mut::<TiledObject>(rectangle).unwrap() = TiledObject::Rectangle {
            width: 10.,
            height: 10.,
        };
        app.update();
        assert_eq!(colliders_area(&app, rectangle), vec![100.]);

        // Removing the TiledObject component despawns its colliders
        app.world_mut()
            .entity_mut(rectangle)
            .remove::<TiledObject>();
        app.update();
        assert!(colliders_area(&app, rectangle).is_empty());
        assert!(app.world().get_entity(rectangle).is_ok());
    }
}

#[cfg(all(feature = "user_properties", feature = "render"))]
mod properties {
    use super::*;

    /// Custom property set on tile 0 of the first tileset of the test map
    #[derive(Component, Reflect, Default, Debug, PartialEq)]
    #[reflect(Component, Default)]
    pub struct TestTileProperty {
        value: i32,
    }

    /// Values of [`TestTileProperty`] found when receiving [`MapCreated`]
    #[derive(Resource, Default)]
    struct PropertiesOnMapCreated(Option<Vec<(String, i32)>>);

    fn property_value(app: &App, entity: Entity) -> Option<i32> {
        app.world().get::<TestTileProperty>(entity).map(|p| p.value)
    }

    fn properties_test_app() -> App {
        let mut app = test_app();
        app.register_type::<TestTileProperty>();
        app.add_plugins(crate::tiled::properties::plugin);
        app.init_resource::<PropertiesOnMapCreated>();
        app.add_observer(
            |_: On<TiledEvent<MapCreated>>,
             mut found: ResMut<PropertiesOnMapCreated>,
             query: Query<(&Name, &TestTileProperty)>| {
                let mut values = query
                    .iter()
                    .map(|(name, p)| (name.to_string(), p.value))
                    .collect::<Vec<_>>();
                values.sort();
                found.0 = Some(values);
            },
        );
        app
    }

    #[test]
    fn tile_properties_are_inserted_before_map_created() {
        let mut app = properties_test_app();
        spawn_test_map(&mut app);

        // Both tiles using tile 0 and both tile objects referencing it have the property,
        // and the object own property takes precedence over the tile one
        assert_eq!(
            app.world().resource::<PropertiesOnMapCreated>().0,
            Some(vec![
                ("Tile(tile_object)".to_string(), 42),
                ("Tile(tile_object_with_override)".to_string(), 7),
                ("TiledMapTile(0,3)".to_string(), 42),
                ("TiledMapTile(1,3)".to_string(), 42),
            ])
        );

        // Properties are not inserted again afterwards
        let tile_object = object_by_name(&mut app, "tile_object_with_override");
        assert_eq!(property_value(&app, tile_object), Some(7));
    }

    #[test]
    fn tile_properties_are_inserted_on_new_tiles_and_tile_objects() {
        let mut app = properties_test_app();
        let map = spawn_test_map(&mut app);
        let ground = layer_by_name(&mut app, "ground");
        let objects = layer_by_name(&mut app, "objects");

        let tile = app
            .world_mut()
            .spawn((
                TiledTile,
                ChildOf(ground),
                TiledTilesetId(0),
                TiledTileId(0),
                TilePos::new(2, 2),
            ))
            .id();
        let other_tile = app
            .world_mut()
            .spawn((
                TiledTile,
                ChildOf(ground),
                TiledTilesetId(0),
                TiledTileId(1),
                TilePos::new(3, 3),
            ))
            .id();
        let tile_object = app
            .world_mut()
            .spawn((
                TiledObject::Tile {
                    width: 16.,
                    height: 16.,
                    tileset_id: 0,
                    tile_id: 0,
                    flip_x: false,
                    flip_y: false,
                },
                ChildOf(objects),
                TiledMapReference(map),
            ))
            .id();
        app.update();

        assert_eq!(property_value(&app, tile), Some(42));
        assert_eq!(property_value(&app, other_tile), None);
        assert_eq!(property_value(&app, tile_object), Some(42));
    }
}

/// Performance measurements on a large map, ignored by default.
///
/// Run with:
/// `cargo test --release --all-features --lib perf -- --ignored --nocapture`
mod perf {
    use super::*;

    const LARGE_MAP: &str = "tests/perf/large_map.tmx";

    /// Minimal deterministic pseudo-random generator
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self, max: u32) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 33) % max as u64) as u32
        }
    }

    /// Run a single frame and return its duration
    fn timed_update(app: &mut App) -> Duration {
        let start = Instant::now();
        app.update();
        start.elapsed()
    }

    /// Run several frames, applying `modify` before each of them, and return the average frame duration
    fn average_frame(app: &mut App, frames: u32, mut modify: impl FnMut(&mut App)) -> Duration {
        let mut total = Duration::ZERO;
        for _ in 0..frames {
            modify(app);
            total += timed_update(app);
        }
        total / frames
    }

    fn layer_tiles_entities(app: &mut App, layer: Entity) -> Vec<Entity> {
        app.world_mut()
            .query_filtered::<(Entity, &ChildOf), With<TiledTile>>()
            .iter(app.world())
            .filter(|(_, c)| c.parent() == layer)
            .map(|(e, _)| e)
            .collect()
    }

    #[test]
    #[ignore = "performance test: cargo test --release --all-features --lib perf -- --ignored --nocapture"]
    fn perf_large_map() {
        let mut app = test_app();
        #[cfg(feature = "physics")]
        app.add_plugins(TiledPhysicsPlugin::<physics::TestBackend>::default());

        // Load the asset first: we do not want to measure IO and parsing
        let handle = app
            .world()
            .resource::<AssetServer>()
            .load::<TiledMapAsset>(LARGE_MAP);
        let deadline = Instant::now() + Duration::from_secs(60);
        while !app
            .world()
            .resource::<AssetServer>()
            .is_loaded_with_dependencies(&handle)
        {
            assert!(Instant::now() < deadline, "large map did not load in time");
            app.update();
            std::thread::sleep(Duration::from_millis(1));
        }

        // Spawn: map data, then rendering and physics components
        app.world_mut().spawn(TiledMap(handle));
        let mut spawn = Duration::ZERO;
        while app
            .world()
            .resource::<Received>()
            .storage_on_map_created
            .is_none()
        {
            spawn += timed_update(&mut app);
        }
        let tiles = app
            .world()
            .resource::<Received>()
            .storage_on_map_created
            .unwrap()
            .1;
        let first_frame = timed_update(&mut app);

        let ground = layer_by_name(&mut app, "ground");
        let decoration = layer_by_name(&mut app, "decoration");
        let mut rng = Lcg(42);

        // Frame without any modification
        let idle = average_frame(&mut app, 20, |_| {});

        // Change the tile ID of 100 ground tiles per frame
        let ground_tiles = layer_tiles_entities(&mut app, ground);
        let change_ids = average_frame(&mut app, 20, |app| {
            for _ in 0..100 {
                let tile = ground_tiles[rng.next(ground_tiles.len() as u32) as usize];
                let mut tile_id = app.world_mut().get_mut::<TiledTileId>(tile).unwrap();
                tile_id.0 = (tile_id.0 + 1) % 4;
            }
        });

        // Move 100 decoration tiles per frame (some of them replace other tiles)
        let move_tiles = average_frame(&mut app, 20, |app| {
            let decoration_tiles = layer_tiles_entities(app, decoration);
            for _ in 0..100 {
                let tile = decoration_tiles[rng.next(decoration_tiles.len() as u32) as usize];
                if let Some(mut pos) = app.world_mut().get_mut::<TilePos>(tile) {
                    *pos = TilePos::new(rng.next(200), rng.next(200));
                }
            }
        });

        // Despawn 1000 ground tiles in a single frame
        for tile in layer_tiles_entities(&mut app, ground)
            .into_iter()
            .take(1000)
        {
            app.world_mut().entity_mut(tile).despawn();
        }
        let despawn = timed_update(&mut app);

        eprintln!("=== Performance on a 200x200 map with 2 tiles layers ({tiles} tiles)");
        eprintln!("spawn (until MapCreated)       : {spawn:?}");
        eprintln!("first frame (render + physics) : {first_frame:?}");
        eprintln!("idle frame                     : {idle:?}");
        eprintln!("change 100 tile IDs / frame    : {change_ids:?}");
        eprintln!("move 100 tiles / frame         : {move_tiles:?}");
        eprintln!("despawn 1000 tiles             : {despawn:?}");
    }
}
