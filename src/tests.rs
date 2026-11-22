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
    entities_on_map_created: Option<(usize, usize, usize)>,
}

/// Build an [`App`] with all the plugin internals but without bevy_ecs_tilemap
/// [`TilemapPlugin`](bevy_ecs_tilemap::TilemapPlugin), which requires a renderer.
pub(crate) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    // Without the `render` feature, loading a map must not require any image related plugin
    #[cfg(feature = "render")]
    {
        app.add_plugins((
            bevy::image::ImagePlugin::default(),
            bevy::image::TextureAtlasPlugin,
        ));
        // Image loader is usually registered by the render plugin
        app.register_asset_loader(bevy::image::ImageLoader::new(
            bevy::image::CompressedImageFormats::empty(),
        ));
    }
    app.add_plugins((
        crate::asset::plugin,
        crate::ecs::plugin,
        crate::properties::plugin,
        #[cfg(feature = "render")]
        crate::render::plugin,
        crate::sets::plugin,
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
         layers: TiledMapLayers<Entity>,
         tiles: TiledMapTiles<Entity>,
         objects: TiledMapObjects<Entity>| {
            received.entities_on_map_created = Some((
                layers.iter(ev.origin).count(),
                tiles.iter(ev.origin).count(),
                objects.iter(ev.origin).count(),
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
    spawn_map_with(app, path, ())
}

/// Spawn the given map with additional components and run the app until it is spawned
fn spawn_map_with(app: &mut App, path: &str, bundle: impl Bundle) -> Entity {
    let handle = app
        .world()
        .resource::<AssetServer>()
        .load::<TiledMapAsset>(path.to_owned());
    let map_entity = app
        .world_mut()
        .spawn((TiledMap(handle.clone()), bundle))
        .id();

    let deadline = Instant::now() + Duration::from_secs(10);
    while app
        .world()
        .resource::<Received>()
        .entities_on_map_created
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
        .map_or_else(|| panic!("missing layer {name}"), |(e, _)| e)
}

fn tile_at(app: &mut App, layer: Entity, pos: TilePos) -> Entity {
    app.world_mut()
        .query_filtered::<(Entity, &TilePos, &ChildOf), With<TiledTile>>()
        .iter(app.world())
        .find(|(_, p, c)| **p == pos && c.parent() == layer)
        .map_or_else(|| panic!("missing tile at {pos:?}"), |(e, _, _)| e)
}

/// Returns the entities of a map matching the filter, sorted
fn map_entities<F: bevy::ecs::query::QueryFilter>(app: &mut App, map: Entity) -> Vec<Entity> {
    let mut entities = app
        .world_mut()
        .query_filtered::<(Entity, &TiledMapReference), F>()
        .iter(app.world())
        .filter_map(|(e, map_ref)| (map_ref.0 == map).then_some(e))
        .collect::<Vec<_>>();
    entities.sort();
    entities
}

/// Returns the entities of a given tile of a map, sorted
#[cfg(feature = "render")]
fn map_tiles_of_kind(app: &mut App, map: Entity, tile: TiledTile) -> Vec<Entity> {
    let mut entities = app
        .world_mut()
        .query::<(Entity, &TiledTile, &TiledMapReference)>()
        .iter(app.world())
        .filter_map(|(e, t, map_ref)| (map_ref.0 == map && *t == tile).then_some(e))
        .collect::<Vec<_>>();
    entities.sort();
    entities
}

#[cfg(feature = "render")]
/// Returns the tilemap of a layer for a given tileset ID
fn tilemap_of(app: &App, layer: Entity, tileset_id: TilesetId) -> Option<Entity> {
    let world = app.world();
    world
        .get::<TiledTilemaps>(layer)?
        .iter()
        .find(|e| world.get::<TiledTilemap>(*e).map(|t| t.tileset_id) == Some(tileset_id))
}

#[cfg(feature = "render")]
fn tilemaps_count(app: &App, layer: Entity) -> usize {
    app.world()
        .get::<TiledTilemaps>(layer)
        .map_or(0, |t| t.len())
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
        .map_or_else(|| panic!("missing object {name}"), |(e, _)| e)
}

#[test]
#[cfg(feature = "render")]
fn map_load_spawns_data_and_render_components() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);

    // Created events are only sent once all the map data have been spawned
    assert_eq!(
        app.world().resource::<Received>().entities_on_map_created,
        Some((3, 4, 3))
    );

    let ground = layer_by_name(&mut app, "ground");
    let decoration = layer_by_name(&mut app, "decoration");

    // Tiles and tilemaps are both direct children of the layer
    let tile = tile_at(&mut app, ground, TilePos::new(0, 3));
    let tilemap = tilemap_of(&app, ground, TilesetId(0)).expect("missing tilemap");
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
    let tilemap = tilemap_of(&app, ground, TilesetId(1)).expect("missing tilemap");
    app.world_mut().entity_mut(tile).despawn();
    app.update();

    // Tilemap does not contain any tile anymore: it has been despawned
    assert!(app.world().get_entity(tilemap).is_err());
    assert_eq!(tilemaps_count(&app, ground), 1);
    assert!(map_tiles_of_kind(&mut app, map, TiledTile::new(TilesetId(1), 1)).is_empty());

    // Despawn a tile which shares its tilemap with another one
    let tile = tile_at(&mut app, ground, TilePos::new(0, 3));
    let tilemap = tilemap_of(&app, ground, TilesetId(0)).expect("missing tilemap");
    app.world_mut().entity_mut(tile).despawn();
    app.update();
    assert_eq!(storage_at(&app, tilemap, TilePos::new(0, 3)), None);
    assert!(storage_at(&app, tilemap, TilePos::new(1, 3)).is_some());
    assert_eq!(
        map_tiles_of_kind(&mut app, map, TiledTile::new(TilesetId(0), 0)).len(),
        1
    );
}

#[test]
#[cfg(feature = "render")]
fn modifying_tiles_updates_tilemaps_and_storage() {
    let mut app = test_app();
    spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    let tilemap_0 = tilemap_of(&app, ground, TilesetId(0)).unwrap();
    let tilemap_1 = tilemap_of(&app, ground, TilesetId(1)).unwrap();

    // Move a tile
    let tile = tile_at(&mut app, ground, TilePos::new(0, 3));
    *app.world_mut().get_mut::<TilePos>(tile).unwrap() = TilePos::new(2, 2);
    app.update();
    assert_eq!(storage_at(&app, tilemap_0, TilePos::new(0, 3)), None);
    assert_eq!(storage_at(&app, tilemap_0, TilePos::new(2, 2)), Some(tile));

    // Change its tile ID
    app.world_mut().get_mut::<TiledTile>(tile).unwrap().tile_id = 3;
    app.update();
    assert_eq!(
        app.world().get::<TileTextureIndex>(tile),
        Some(&TileTextureIndex(3))
    );

    // Change its tileset: it moves to the other tilemap of the layer
    app.world_mut()
        .get_mut::<TiledTile>(tile)
        .unwrap()
        .tileset_id = TilesetId(1);
    app.update();
    assert_eq!(
        app.world().get::<TilemapId>(tile),
        Some(&TilemapId(tilemap_1))
    );
    assert_eq!(storage_at(&app, tilemap_0, TilePos::new(2, 2)), None);
    assert_eq!(storage_at(&app, tilemap_1, TilePos::new(2, 2)), Some(tile));
}

#[test]
#[cfg(feature = "render")]
fn spawning_tiles_creates_tilemaps() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);
    let decoration = layer_by_name(&mut app, "decoration");
    assert!(tilemap_of(&app, decoration, TilesetId(1)).is_none());

    // Only the data components are needed to spawn a new tile
    let tile = app
        .world_mut()
        .spawn((
            TiledTile::new(TilesetId(1), 2),
            ChildOf(decoration),
            TilePos::new(2, 2),
        ))
        .id();
    app.update();

    let tilemap = tilemap_of(&app, decoration, TilesetId(1)).expect("tilemap was not created");
    assert_eq!(storage_at(&app, tilemap, TilePos::new(2, 2)), Some(tile));
    assert_eq!(
        app.world().resource::<Received>().tilemaps_created.last(),
        Some(&tilemap)
    );
    assert_eq!(app.world().get::<TiledMapReference>(tile).unwrap().0, map);
    assert_eq!(
        map_tiles_of_kind(&mut app, map, TiledTile::new(TilesetId(1), 2)),
        vec![tile]
    );
}

#[test]
#[cfg(feature = "render")]
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

#[test]
fn tiles_of_different_layers_at_the_same_position_are_kept() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    let decoration = layer_by_name(&mut app, "decoration");
    let ground_tile = tile_at(&mut app, ground, TilePos::new(0, 3));

    // Spawning a tile in another layer, at the same position, does not replace it
    let decoration_tile = app
        .world_mut()
        .spawn((
            TiledTile::new(TilesetId(0), 1),
            ChildOf(decoration),
            TilePos::new(0, 3),
        ))
        .id();
    app.update();
    assert!(app.world().get_entity(ground_tile).is_ok());
    let tile_of = |layer| {
        app.world()
            .get::<TiledTileLayerStorage>(layer)
            .unwrap()
            .get(TilePos::new(0, 3))
    };
    assert_eq!(tile_of(ground), Some(ground_tile));
    assert_eq!(tile_of(decoration), Some(decoration_tile));
    assert_eq!(layer_index(&app, ground), layer_tiles(&mut app, ground));
    assert_eq!(
        layer_index(&app, decoration),
        layer_tiles(&mut app, decoration)
    );

    // Neither does moving a tile of another layer onto it
    *app.world_mut().get_mut::<TilePos>(decoration_tile).unwrap() = TilePos::new(1, 3);
    app.update();
    let other_ground_tile = tile_at(&mut app, ground, TilePos::new(1, 3));
    *app.world_mut().get_mut::<TilePos>(decoration_tile).unwrap() = TilePos::new(0, 3);
    app.update();
    assert!(app.world().get_entity(ground_tile).is_ok());
    assert!(app.world().get_entity(other_ground_tile).is_ok());
    assert_eq!(layer_index(&app, ground), layer_tiles(&mut app, ground));
    assert_eq!(
        layer_index(&app, decoration),
        layer_tiles(&mut app, decoration)
    );

    // Tiles of a tiles layer spawned at runtime are indexed as well
    let layer = app
        .world_mut()
        .spawn((TiledLayer::Tiles, TiledMapReference(map), ChildOf(map)))
        .id();
    let tile = app
        .world_mut()
        .spawn((
            TiledTile::new(TilesetId(0), 1),
            ChildOf(layer),
            TilePos::new(2, 2),
        ))
        .id();
    app.update();
    assert_eq!(layer_index(&app, layer), vec![(TilePos::new(2, 2), tile)]);
}

/// Tiles of a layer, as registered in its storage
fn layer_index(app: &App, layer: Entity) -> Vec<(TilePos, Entity)> {
    let storage = app.world().get::<TiledTileLayerStorage>(layer).unwrap();
    let mut tiles = storage.iter().collect::<Vec<_>>();
    assert_eq!(tiles.len(), storage.len());
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
    spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    let a = tile_at(&mut app, ground, TilePos::new(0, 3));
    let b = tile_at(&mut app, ground, TilePos::new(1, 3));

    // Move A onto B: B is despawned
    *app.world_mut().get_mut::<TilePos>(a).unwrap() = TilePos::new(1, 3);
    app.update();
    assert!(app.world().get_entity(b).is_err());
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
        let tilemap = tilemap_of(&app, ground, TilesetId(0)).unwrap();
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
            TiledTile::new(TilesetId(1), 2),
            ChildOf(ground),
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
            TiledTile::new(TilesetId(0), 1),
            ChildOf(ground),
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
    use crate::render::tilemap::TiledTileRenderState;

    let mut app = test_app();
    spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    let tilemap = tilemap_of(&app, ground, TilesetId(0)).unwrap();
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
                tileset_id: TilesetId(0),
                tile_id: 1,
                flip_x: false,
                flip_y: false,
            },
            ChildOf(objects),
        ))
        .id();
    app.update();

    assert_eq!(app.world().get::<TiledMapReference>(object).unwrap().0, map);
    assert!(map_entities::<With<TiledObject>>(&mut app, map).contains(&object));
    #[cfg(feature = "render")]
    assert_eq!(
        app.world()
            .get::<TiledObjectVisuals>(object)
            .map(|v| v.len()),
        Some(1)
    );
}

#[test]
fn map_entities_are_respawned_with_the_map() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);
    let mut layers = ["ground", "decoration", "objects"].map(|n| layer_by_name(&mut app, n));
    let mut objects = ["tile_object", "tile_object_with_override", "rectangle"]
        .map(|n| object_by_name(&mut app, n));
    layers.sort();
    objects.sort();
    assert_eq!(map_entities::<With<TiledLayer>>(&mut app, map), layers);
    assert_eq!(map_entities::<With<TiledObject>>(&mut app, map), objects);

    // Respawning the map despawns the previous entities
    app.world_mut().entity_mut(map).insert(RespawnTiledMap);
    app.update();
    app.update();
    let new_layers = map_entities::<With<TiledLayer>>(&mut app, map);
    let new_objects = map_entities::<With<TiledObject>>(&mut app, map);
    assert_eq!(new_layers.len(), 3);
    assert_eq!(new_objects.len(), 3);
    assert!(new_layers.iter().all(|e| !layers.contains(e)));
    assert!(new_objects.iter().all(|e| !objects.contains(e)));
    assert!(layers.iter().all(|e| app.world().get_entity(*e).is_err()));
}

#[test]
fn tiled_properties_are_inserted() {
    let mut app = test_app();
    let map = spawn_test_map(&mut app);
    let decoration = layer_by_name(&mut app, "decoration");
    let ground = layer_by_name(&mut app, "ground");
    let rectangle = object_by_name(&mut app, "rectangle");
    let tile_object = object_by_name(&mut app, "tile_object");
    let world = app.world();

    let properties = world.get::<TiledProperties>(decoration).unwrap();
    assert_eq!(properties.class, "TestLayerClass");
    assert_eq!(
        properties.properties.get("test_layer_property"),
        Some(&tiled::PropertyValue::StringValue(
            "layer value".to_string()
        ))
    );

    let properties = world.get::<TiledProperties>(rectangle).unwrap();
    assert_eq!(properties.class, "TestObjectClass");
    assert_eq!(
        properties.properties.get("test_object_property"),
        Some(&tiled::PropertyValue::IntValue(3))
    );

    // Entities without class nor custom property also get an empty component
    for entity in [map, ground, tile_object] {
        let properties = world.get::<TiledProperties>(entity).unwrap();
        assert!(properties.class.is_empty());
        assert!(properties.properties.is_empty());
    }

    // The component is registered for reflection
    let registry = world.resource::<AppTypeRegistry>().read();
    let registration = registry
        .get(std::any::TypeId::of::<TiledProperties>())
        .expect("TiledProperties is not registered");
    assert!(registration.data::<ReflectComponent>().is_some());
}

#[test]
fn id_types_are_registered() {
    let app = test_app();
    let registry = app.world().resource::<AppTypeRegistry>().read();
    for (name, type_id) in [
        ("TilesetId", std::any::TypeId::of::<TilesetId>()),
        ("LayerId", std::any::TypeId::of::<LayerId>()),
        ("ObjectId", std::any::TypeId::of::<ObjectId>()),
    ] {
        assert!(registry.get(type_id).is_some(), "{name} is not registered");
    }
}

/// Check that the tile count of each tilemap matches the content of its storage
#[cfg(feature = "render")]
fn assert_tilemaps_count_match_storage(app: &mut App) {
    use crate::render::tilemap::TiledTilemapTileCount;

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

/// Check that the layers storages match the actual tiles
fn assert_storage_matches_tiles(app: &mut App) {
    // Layers storages hold exactly the tiles of each layer, at their position
    let layers = app
        .world_mut()
        .query_filtered::<Entity, With<TiledLayer>>()
        .iter(app.world())
        .collect::<Vec<_>>();
    for layer in layers {
        assert_eq!(layer_index(app, layer), layer_tiles(app, layer));
    }

    #[cfg(feature = "render")]
    assert_tilemaps_count_match_storage(app);
}

#[test]
fn storage_indexes_tiles_positions() {
    let mut app = test_app();
    spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    let decoration = layer_by_name(&mut app, "decoration");
    assert_storage_matches_tiles(&mut app);

    let a = tile_at(&mut app, ground, TilePos::new(0, 3));
    let b = tile_at(&mut app, ground, TilePos::new(1, 3));
    let tiles_at = |app: &App, x, y| {
        let mut tiles = [ground, decoration]
            .into_iter()
            .filter_map(|layer| {
                app.world()
                    .get::<TiledTileLayerStorage>(layer)
                    .unwrap()
                    .get(TilePos::new(x, y))
            })
            .collect::<Vec<_>>();
        tiles.sort();
        tiles
    };
    assert_eq!(tiles_at(&app, 0, 3), vec![a]);

    // Tiles of different layers share the same position
    let c = app
        .world_mut()
        .spawn((
            TiledTile::new(TilesetId(1), 2),
            ChildOf(decoration),
            TilePos::new(0, 3),
        ))
        .id();
    app.update();
    let mut expected = vec![a, c];
    expected.sort();
    assert_eq!(tiles_at(&app, 0, 3), expected);
    assert_storage_matches_tiles(&mut app);

    // Moving B onto A replaces A, but not C which is in another layer
    *app.world_mut().get_mut::<TilePos>(b).unwrap() = TilePos::new(0, 3);
    app.update();
    let mut expected = vec![b, c];
    expected.sort();
    assert_eq!(tiles_at(&app, 0, 3), expected);
    assert!(tiles_at(&app, 1, 3).is_empty());
    assert_storage_matches_tiles(&mut app);

    // Despawning a tile removes it from its position
    app.world_mut().entity_mut(c).despawn();
    app.update();
    assert_eq!(tiles_at(&app, 0, 3), vec![b]);
    assert_storage_matches_tiles(&mut app);

    // Removing the TiledTile component as well
    app.world_mut().entity_mut(b).remove::<TiledTile>();
    app.update();
    assert!(tiles_at(&app, 0, 3).is_empty());
    assert_storage_matches_tiles(&mut app);
}

#[test]
fn storage_follows_tiles() {
    let mut app = test_app();
    spawn_test_map(&mut app);
    let ground = layer_by_name(&mut app, "ground");
    assert_storage_matches_tiles(&mut app);

    // Several tiles sharing the same tileset ID + tile ID
    let tiles = (0..6)
        .map(|i| {
            app.world_mut()
                .spawn((
                    TiledTile::new(TilesetId(0), 0),
                    ChildOf(ground),
                    TilePos::new(i % 4, 1 + i / 4),
                ))
                .id()
        })
        .collect::<Vec<_>>();
    app.update();
    assert_storage_matches_tiles(&mut app);

    // Despawn some of them and change the tile ID of others
    app.world_mut().entity_mut(tiles[0]).despawn();
    app.world_mut().entity_mut(tiles[3]).despawn();
    app.world_mut()
        .get_mut::<TiledTile>(tiles[1])
        .unwrap()
        .tile_id = 2;
    app.world_mut()
        .get_mut::<TiledTile>(tiles[5])
        .unwrap()
        .tileset_id = TilesetId(1);
    app.update();
    assert_storage_matches_tiles(&mut app);

    app.world_mut().entity_mut(tiles[2]).despawn();
    app.world_mut()
        .get_mut::<TiledTile>(tiles[1])
        .unwrap()
        .tile_id = 0;
    app.update();
    assert_storage_matches_tiles(&mut app);
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
                TiledTile::new(TilesetId(0), 0),
                ChildOf(layer),
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

/// Image collection tilesets are skipped with the `atlas` feature, whether rendering or not
#[test]
fn image_collection_tilesets_only_depend_on_atlas_feature() {
    let mut app = test_app();
    let map = spawn_map(&mut app, "maps/orthogonal/finite.tmx");

    let handle = app.world().get::<TiledMap>(map).unwrap().0.clone();
    let assets = app.world().resource::<Assets<TiledMapAsset>>();
    let asset = assets.get(&handle).unwrap();
    assert_eq!(
        asset.tileset(TilesetId(0)).is_some(),
        cfg!(not(feature = "atlas"))
    );
}

/// The position of an image does not depend on rendering, except along
/// a repeated axis where it also depends on what the camera can see
#[test]
fn image_transform_follows_base_position() {
    let mut app = test_app();
    spawn_map(&mut app, "maps/orthogonal/finite.tmx");

    let (image, transform) = app
        .world_mut()
        .query::<(&TiledImage, &Transform)>()
        .single(app.world())
        .unwrap();
    assert!(!image.repeat_x);
    assert_ne!(image.base_position.x, 0.);
    assert_eq!(transform.translation.x, image.base_position.x);
}

/// Minimal set of plugins needed to load and spawn a map without the `render` feature
#[test]
#[cfg(not(feature = "render"))]
fn tiled_plugin_only_requires_asset_plugin() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TiledPlugin::default(),
    ));
    let handle = app
        .world()
        .resource::<AssetServer>()
        .load::<TiledMapAsset>("maps/orthogonal/finite.tmx");
    let map = app.world_mut().spawn(TiledMap(handle)).id();

    let deadline = Instant::now() + Duration::from_secs(10);
    while app
        .world_mut()
        .query_filtered::<&TiledMapReference, With<TiledLayer>>()
        .iter(app.world())
        .all(|map_ref| map_ref.0 != map)
    {
        assert!(Instant::now() < deadline, "test map did not spawn in time");
        app.update();
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
#[cfg(not(feature = "render"))]
fn text_objects_are_not_displayed_without_render() {
    let mut app = test_app();
    spawn_map(&mut app, "maps/orthogonal/text.tmx");

    // Text data is still available...
    let texts = app
        .world_mut()
        .query::<&TiledObject>()
        .iter(app.world())
        .filter_map(|object| match object {
            TiledObject::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!texts.is_empty());
    assert!(texts.iter().all(|text| text == "Hello World"));

    // ...but nothing is displayed: objects have no child visual entity
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), (With<TiledObject>, With<Children>)>()
            .iter(app.world())
            .count(),
        0
    );
}

#[test]
#[cfg(not(feature = "render"))]
fn map_is_spawned_without_image_assets() {
    let mut app = test_app();
    spawn_map(&mut app, "maps/orthogonal/finite.tmx");

    assert!(!app.world().contains_resource::<Assets<Image>>());
    assert!(
        !app.world()
            .contains_resource::<Assets<TextureAtlasLayout>>()
    );

    // Image data is still available, but nothing is rendered
    let (image, transform) = app
        .world_mut()
        .query::<(&TiledImage, &Transform)>()
        .single(app.world())
        .unwrap();
    assert!(image.base_size.x > 0. && image.base_size.y > 0.);
    assert!(image.repeat_y);
    assert_eq!(transform.translation, image.base_position.extend(0.));
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), With<Sprite>>()
            .iter(app.world())
            .count(),
        0
    );

    // Tile objects are spawned without any visual
    assert!(
        app.world_mut()
            .query::<&TiledObject>()
            .iter(app.world())
            .any(|object| matches!(object, TiledObject::Tile { .. }))
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<(), (With<TiledObject>, With<Children>)>()
            .iter(app.world())
            .count(),
        0
    );
}

#[test]
#[cfg(feature = "render")]
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

#[cfg(feature = "render")]
#[test]
fn repeated_images_cover_camera_area_when_map_is_rotated() {
    let mut app = test_app();
    app.add_plugins(bevy::transform::TransformPlugin);
    let map = spawn_test_map(&mut app);
    let layer = layer_by_name(&mut app, "objects");

    // Rotate the map by a quarter turn: the image X axis is now the world Y axis
    app.world_mut()
        .entity_mut(map)
        .insert(Transform::from_rotation(Quat::from_rotation_z(
            core::f32::consts::FRAC_PI_2,
        )));

    // A camera seeing a 200 pixels wide and 2000 pixels high area
    app.world_mut().spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            area: Rect::new(-100., -1000., 100., 1000.),
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
    // Let the map rotation propagate to the layer GlobalTransform
    app.update();
    app.update();

    // The repeated image covers the camera area height, not its width
    let sprite = app.world().get::<Sprite>(image).unwrap();
    let size = sprite.custom_size.unwrap();
    assert!(size.x >= 2000., "repeated image width is {}", size.x);
    assert_eq!(size.y, 16.);
}

/// Move a tile of the given map, then check that it moved in its tilemap and that its collider followed it
#[cfg(any(feature = "render", feature = "physics"))]
fn check_tile_move(path: &str) {
    let mut app = test_app();
    #[cfg(feature = "physics")]
    app.add_plugins(TiledPhysicsPlugin::<physics::TestBackend>::default());
    let _map = spawn_map(&mut app, path);
    let ground = layer_by_name(&mut app, "ground");
    let (from, to) = (TilePos::new(0, 3), TilePos::new(2, 1));
    let tile = tile_at(&mut app, ground, from);

    #[cfg(feature = "physics")]
    let before = physics::colliders_bounds(&app, ground, Vec2::ZERO);

    *app.world_mut().get_mut::<TilePos>(tile).unwrap() = to;
    app.update();

    #[cfg(feature = "render")]
    {
        let tilemap = app.world().get::<TilemapId>(tile).unwrap().0;
        assert_eq!(storage_at(&app, tilemap, to), Some(tile));
        assert_eq!(storage_at(&app, tilemap, from), None);
    }

    #[cfg(feature = "physics")]
    {
        // The collider moves by the same offset than the tile rendering position
        let expected_offset = {
            let handle = &app.world().get::<TiledMap>(_map).unwrap().0;
            let assets = app.world().resource::<Assets<TiledMapAsset>>();
            let map_asset = assets.get(handle).unwrap();
            let tile = TiledTile::new(TilesetId(0), 0);
            let anchor = TilemapAnchor::default();
            map_asset
                .tile_relative_position(&to, &tile, &anchor)
                .unwrap()
                - map_asset
                    .tile_relative_position(&from, &tile, &anchor)
                    .unwrap()
        };
        let after = physics::colliders_bounds(&app, ground, Vec2::ZERO);
        assert_eq!(after.0, before.0, "collider size changed");
        assert!(
            (after.1 - before.1 - expected_offset).length() < 1e-3,
            "collider moved by {:?} instead of {:?}",
            after.1 - before.1,
            expected_offset
        );
    }
}

#[test]
#[cfg(any(feature = "render", feature = "physics"))]
fn moving_tiles_on_isometric_map() {
    check_tile_move("tests/dynamic/isometric_map.tmx");
}

#[test]
#[cfg(any(feature = "render", feature = "physics"))]
fn moving_tiles_on_hexagonal_map() {
    check_tile_move("tests/dynamic/hexagonal_map.tmx");
}

/// Load the given map asset and run the app until it is loaded
fn load_map_asset(app: &mut App, path: &str) -> Handle<TiledMapAsset> {
    let handle = app
        .world()
        .resource::<AssetServer>()
        .load::<TiledMapAsset>(path.to_owned());
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            Instant::now() < deadline,
            "{path}: map did not load in time"
        );
        match app.world().resource::<AssetServer>().load_state(&handle) {
            LoadState::Loaded => return handle,
            LoadState::Failed(error) => panic!("{path}: map failed to load: {error}"),
            _ => {}
        }
        app.update();
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// On hexagonal maps, each tile is rendered where Tiled puts it, whatever the map
/// stagger settings, its size parity and its hexagon side length.
#[test]
fn hexagonal_tiles_match_tiled_positions() {
    let maps = [
        "maps/hexagonal/finite_flat_top_even.tmx",
        "maps/hexagonal/finite_flat_top_odd.tmx",
        "maps/hexagonal/finite_pointy_top_even.tmx",
        "maps/hexagonal/finite_pointy_top_odd.tmx",
        "tests/hexagonal/pointy_top_odd_height_odd.tmx",
        "tests/hexagonal/pointy_top_odd_height_even.tmx",
        "tests/hexagonal/pointy_top_custom_side.tmx",
        "tests/hexagonal/flat_top_custom_side.tmx",
    ];
    let anchors = [
        TilemapAnchor::None,
        TilemapAnchor::Center,
        TilemapAnchor::TopLeft,
    ];
    let mut app = test_app();
    let mut errors = vec![];
    for path in maps {
        let handle = load_map_asset(&mut app, path);
        let assets = app.world().resource::<Assets<TiledMapAsset>>();
        let map_asset = assets.get(&handle).unwrap();
        let map = &map_asset.map;
        let (tw, th) = (map.tile_width as f32, map.tile_height as f32);
        let side = map.hex_side_length.unwrap_or(0) as f32;
        let (w, h) = (map.width, map.height);
        let staggered = |i: u32| (i % 2 == 1) == (map.stagger_index == tiled::StaggerIndex::Odd);
        let shift = |i: u32| if staggered(i) { 0.5 } else { 0. };

        // Map size, as Tiled sees it
        let expected_size = match map.stagger_axis {
            tiled::StaggerAxis::Y => Vec2::new(
                w as f32 * tw + if h > 1 { tw / 2. } else { 0. },
                h as f32 * (th + side) / 2. + (th - side) / 2.,
            ),
            tiled::StaggerAxis::X => Vec2::new(
                w as f32 * (tw + side) / 2. + (tw - side) / 2.,
                h as f32 * th + if w > 1 { th / 2. } else { 0. },
            ),
        };
        if !map_asset.rect.size().abs_diff_eq(expected_size, 0.01) {
            errors.push(format!(
                "{path}: map size is {}, expected {expected_size}",
                map_asset.rect.size()
            ));
        }

        let tile_size = TilemapTileSize::new(tw, th);
        for anchor in anchors {
            for x in 0..w {
                for r in 0..h {
                    // Tile center, in Tiled pixel coordinates
                    let tiled_center = match map.stagger_axis {
                        tiled::StaggerAxis::Y => Vec2::new(
                            (x as f32 + 0.5 + shift(r)) * tw,
                            r as f32 * (th + side) / 2. + th / 2.,
                        ),
                        tiled::StaggerAxis::X => Vec2::new(
                            x as f32 * (tw + side) / 2. + tw / 2.,
                            (r as f32 + 0.5 + shift(x)) * th,
                        ),
                    };
                    let expected = map_asset.world_space_from_tiled_position(&anchor, tiled_center);
                    let actual =
                        map_asset.tile_center(&TilePos::new(x, h - 1 - r), &tile_size, &anchor);
                    if !actual.abs_diff_eq(expected, 0.01) {
                        errors.push(format!(
                            "{path} ({anchor:?}): Tiled tile ({x}, {r}) is at {actual}, expected {expected}"
                        ));
                    }
                }
            }
        }
    }
    assert!(errors.is_empty(), "\n{}", errors.join("\n"));
}

#[cfg(feature = "physics")]
mod physics {
    use super::*;
    use ::geo::{Area, BoundingRect};
    use bevy::platform::collections::HashMap;

    /// Physics backend whose colliders do not hold any physics component
    #[derive(Default, Clone, Debug, Reflect)]
    pub(super) struct TestBackend;

    impl TiledPhysicsBackend for TestBackend {
        type Collider = TestCollider;

        fn colliders(
            &self,
            strategy: TiledColliderStrategy,
            _source: TiledColliderSource,
            geometry: &TiledColliderGeometry,
        ) -> Vec<Self::Collider> {
            // Produce as many colliders as built-in backends
            let count = crate::physics::geometry::collider_shapes(strategy, geometry).len();
            vec![TestCollider(strategy); count]
        }
    }

    /// Collider of the [`TestBackend`], holding the strategy it was created with
    #[derive(Component, Clone, Copy, PartialEq, Debug)]
    pub(super) struct TestCollider(pub(super) TiledColliderStrategy);

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
    pub(super) fn colliders_bounds(app: &App, entity: Entity, origin: Vec2) -> (Vec2, Vec2) {
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
                TiledTile::new(TilesetId(0), 3),
                ChildOf(decoration),
                tile_pos,
            ))
            .id();
        app.update();

        let tile_center = {
            let handle = &app.world().get::<TiledMap>(map).unwrap().0;
            let assets = app.world().resource::<Assets<TiledMapAsset>>();
            assets.get(handle).unwrap().tile_center(
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

    /// Strategies of the colliders of an entity
    fn colliders_strategies(app: &App, entity: Entity) -> Vec<TiledColliderStrategy> {
        app.world()
            .get::<TiledColliders>(entity)
            .into_iter()
            .flat_map(|c| c.iter())
            .map(|e| app.world().get::<TestCollider>(e).unwrap().0)
            .collect()
    }

    /// Number of [`ColliderCreated`] events received for each collider source
    #[derive(Resource, Default)]
    struct CollidersCreated(HashMap<Entity, usize>);

    fn count_colliders_created(app: &mut App) {
        app.init_resource::<CollidersCreated>();
        app.add_observer(
            |ev: On<TiledEvent<ColliderCreated>>, mut created: ResMut<CollidersCreated>| {
                let ev = ev.event();
                // Events are propagated to parent entities: only count the original one
                if ev.is_original() {
                    assert_eq!(ev.source_entity(), ev.entity);
                    *created.0.entry(ev.entity).or_default() += 1;
                }
            },
        );
    }

    fn colliders_created(app: &App, entity: Entity) -> usize {
        app.world()
            .resource::<CollidersCreated>()
            .0
            .get(&entity)
            .copied()
            .unwrap_or(0)
    }

    #[test]
    fn collider_strategy_can_be_overridden() {
        use TiledColliderStrategy::*;

        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        count_colliders_created(&mut app);
        spawn_test_map(&mut app);
        let ground = layer_by_name(&mut app, "ground");
        let objects = layer_by_name(&mut app, "objects");
        let rectangle = object_by_name(&mut app, "rectangle");

        // Map strategy is used by default, and the first collider is the source entity itself
        assert_eq!(colliders_strategies(&app, ground), vec![Polyline]);
        assert_eq!(colliders_strategies(&app, rectangle), vec![Polyline]);
        assert_eq!(
            app.world().get::<TestCollider>(ground),
            Some(&TestCollider(Polyline))
        );
        assert_eq!(
            (
                colliders_created(&app, ground),
                colliders_created(&app, rectangle)
            ),
            (1, 1)
        );

        // Override for a tiles layer: only the colliders of this layer are spawned again
        app.world_mut().entity_mut(ground).insert(Triangulation);
        app.update();
        assert_eq!(colliders_strategies(&app, ground), vec![Triangulation]);
        assert_eq!(colliders_strategies(&app, rectangle), vec![Polyline]);
        assert_eq!(
            (
                colliders_created(&app, ground),
                colliders_created(&app, rectangle)
            ),
            (2, 1)
        );

        // Override for an objects layer applies to its objects
        app.world_mut().entity_mut(objects).insert(LineStrip);
        app.update();
        assert_eq!(colliders_strategies(&app, rectangle), vec![LineStrip]);
        assert_eq!(
            (
                colliders_created(&app, ground),
                colliders_created(&app, rectangle)
            ),
            (2, 2)
        );

        // Override for an object takes precedence over the one of its layer
        app.world_mut().entity_mut(rectangle).insert(Triangulation);
        app.update();
        assert_eq!(colliders_strategies(&app, rectangle), vec![Triangulation]);

        // Removing overrides falls back to the layer strategy, then to the map one
        app.world_mut()
            .entity_mut(rectangle)
            .remove::<TiledColliderStrategy>();
        app.update();
        assert_eq!(colliders_strategies(&app, rectangle), vec![LineStrip]);
        app.world_mut()
            .entity_mut(objects)
            .remove::<TiledColliderStrategy>();
        app.world_mut()
            .entity_mut(ground)
            .remove::<TiledColliderStrategy>();
        app.update();
        assert_eq!(colliders_strategies(&app, rectangle), vec![Polyline]);
        assert_eq!(colliders_strategies(&app, ground), vec![Polyline]);
        assert_eq!(colliders_created(&app, ground), 3);
    }

    #[test]
    fn disabled_strategy_selects_which_sources_get_colliders() {
        use TiledColliderStrategy::*;

        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        let map = spawn_test_map(&mut app);
        let ground = layer_by_name(&mut app, "ground");
        let objects = layer_by_name(&mut app, "objects");
        let rectangle = object_by_name(&mut app, "rectangle");
        assert_eq!(colliders_strategies(&app, ground), vec![Polyline]);
        assert_eq!(colliders_strategies(&app, rectangle), vec![Polyline]);

        // Disable a tiles layer and an objects layer: their colliders are despawned
        app.world_mut().entity_mut(ground).insert(Disabled);
        app.world_mut().entity_mut(objects).insert(Disabled);
        app.update();
        assert!(colliders_strategies(&app, ground).is_empty());
        assert!(colliders_strategies(&app, rectangle).is_empty());
        assert!(app.world().get::<TestCollider>(ground).is_none());
        assert!(app.world().get::<TestCollider>(rectangle).is_none());

        // An object of a disabled layer can still have colliders
        app.world_mut().entity_mut(rectangle).insert(Triangulation);
        app.update();
        assert_eq!(colliders_strategies(&app, rectangle), vec![Triangulation]);

        // Removing the overrides restores the colliders
        app.world_mut()
            .entity_mut(ground)
            .remove::<TiledColliderStrategy>();
        app.world_mut()
            .entity_mut(objects)
            .remove::<TiledColliderStrategy>();
        app.world_mut()
            .entity_mut(rectangle)
            .remove::<TiledColliderStrategy>();
        app.update();
        assert_eq!(colliders_strategies(&app, ground), vec![Polyline]);
        assert_eq!(colliders_strategies(&app, rectangle), vec![Polyline]);

        // Disable colliders for the whole map, but keep them for a single layer
        app.world_mut()
            .get_mut::<TiledPhysicsSettings<TestBackend>>(map)
            .unwrap()
            .collider_strategy = Disabled;
        app.world_mut().entity_mut(ground).insert(Polyline);
        app.update();
        assert_eq!(colliders_strategies(&app, ground), vec![Polyline]);
        assert!(colliders_strategies(&app, rectangle).is_empty());
    }

    #[test]
    fn map_collider_strategy_is_used() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        spawn_map_with(
            &mut app,
            DYNAMIC_MAP,
            TiledPhysicsSettings::<TestBackend> {
                collider_strategy: TiledColliderStrategy::Triangulation,
                ..default()
            },
        );

        let ground = layer_by_name(&mut app, "ground");
        let rectangle = object_by_name(&mut app, "rectangle");
        assert_eq!(
            colliders_strategies(&app, ground),
            vec![TiledColliderStrategy::Triangulation]
        );
        assert_eq!(
            colliders_strategies(&app, rectangle),
            vec![TiledColliderStrategy::Triangulation]
        );
    }

    /// Collider chunks of a layer: chunk index, chunk entity and area of its colliders
    fn collider_chunks(app: &mut App, layer: Entity) -> Vec<((u32, u32), Entity, f32)> {
        let mut chunks = app
            .world_mut()
            .query::<(Entity, &TiledColliderChunk, &ChildOf, &TiledColliderOf)>()
            .iter(app.world())
            .filter(|(_, _, child_of, collider_of)| {
                child_of.parent() == layer && collider_of.0 == layer
            })
            .map(|(entity, chunk, _, _)| {
                let area = app
                    .world()
                    .get::<TiledColliderPolygons>(entity)
                    .unwrap()
                    .iter()
                    .map(|p| p.unsigned_area())
                    .sum();
                ((chunk.0.x, chunk.0.y), entity, area)
            })
            .collect::<Vec<_>>();
        chunks.sort_by_key(|(chunk, _, _)| *chunk);
        // When there are chunks, they are the only colliders of the layer
        if !chunks.is_empty() {
            assert_eq!(
                app.world().get::<TiledColliders>(layer).unwrap().len(),
                chunks.len()
            );
        }
        chunks
    }

    #[test]
    fn tiles_layers_colliders_are_split_in_chunks() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        spawn_map_with(
            &mut app,
            DYNAMIC_MAP,
            TiledPhysicsSettings::<TestBackend> {
                tiles_chunk_size: TiledColliderChunkSize::Size(UVec2::ONE),
                ..default()
            },
        );
        let ground = layer_by_name(&mut app, "ground");

        // Each of the two tiles is in its own chunk, and the layer itself does not hold any collider
        let chunks = collider_chunks(&mut app, ground);
        assert_eq!(
            chunks.iter().map(|(c, _, a)| (*c, *a)).collect::<Vec<_>>(),
            vec![((0, 3), 256.), ((1, 3), 256.)]
        );
        assert!(app.world().get::<TestCollider>(ground).is_none());
        let (first, second) = (chunks[0].1, chunks[1].1);

        // Modifying a tile spawns again the colliders of its chunk, and of the adjacent chunks
        // since their outline depends on it
        let tile = tile_at(&mut app, ground, TilePos::new(1, 3));
        app.world_mut().get_mut::<TileFlip>(tile).unwrap().x = true;
        app.update();
        let chunks = collider_chunks(&mut app, ground);
        assert_eq!(chunks.len(), 2);
        assert_ne!(chunks[0].1, first);
        assert_ne!(chunks[1].1, second);
        assert!(app.world().get_entity(first).is_err());
        assert!(app.world().get_entity(second).is_err());
        let (first, second) = (chunks[0].1, chunks[1].1);

        // Moving a tile updates both the chunk it leaves and the one it enters,
        // and the chunks they are next to
        let tile = tile_at(&mut app, ground, TilePos::new(0, 3));
        *app.world_mut().get_mut::<TilePos>(tile).unwrap() = TilePos::new(2, 1);
        app.update();
        let chunks = collider_chunks(&mut app, ground);
        assert_eq!(
            chunks.iter().map(|(c, _, a)| (*c, *a)).collect::<Vec<_>>(),
            vec![((1, 3), 256.), ((2, 1), 256.)]
        );
        assert!(app.world().get_entity(first).is_err());
        assert!(app.world().get_entity(second).is_err());
        let second = chunks[0].1;

        // Despawning a tile removes its chunk: other chunks are not next to it and are kept
        app.world_mut().entity_mut(tile).despawn();
        app.update();
        let chunks = collider_chunks(&mut app, ground);
        assert_eq!(
            chunks.iter().map(|(c, _, _)| *c).collect::<Vec<_>>(),
            vec![(1, 3)]
        );
        assert_eq!(chunks[0].1, second);

        // Spawning a tile creates one
        app.world_mut().spawn((
            TiledTile::new(TilesetId(0), 0),
            ChildOf(ground),
            TilePos::new(3, 0),
        ));
        app.update();
        let chunks = collider_chunks(&mut app, ground);
        assert_eq!(
            chunks.iter().map(|(c, _, _)| *c).collect::<Vec<_>>(),
            vec![(1, 3), (3, 0)]
        );
        assert_eq!(chunks[0].1, second);
    }

    #[test]
    fn disabled_strategy_on_tile_excludes_it_from_layer_colliders() {
        use TiledColliderStrategy::*;

        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        count_colliders_created(&mut app);
        spawn_test_map(&mut app);
        let ground = layer_by_name(&mut app, "ground");
        let tile = tile_at(&mut app, ground, TilePos::new(1, 3));
        assert_eq!(colliders_area(&app, ground), vec![512.]);

        // A disabled tile is not part of the layer colliders
        app.world_mut().entity_mut(tile).insert(Disabled);
        app.update();
        assert_eq!(colliders_area(&app, ground), vec![256.]);
        assert_eq!(colliders_strategies(&app, ground), vec![Polyline]);
        assert_eq!(colliders_created(&app, ground), 2);

        // Other strategies have no effect on a tile
        app.world_mut().entity_mut(tile).insert(Triangulation);
        app.update();
        assert_eq!(colliders_area(&app, ground), vec![512.]);
        assert_eq!(colliders_strategies(&app, ground), vec![Polyline]);

        // Removing the strategy restores the tile collider
        app.world_mut().entity_mut(tile).insert(Disabled);
        app.update();
        assert_eq!(colliders_area(&app, ground), vec![256.]);
        app.world_mut()
            .entity_mut(tile)
            .remove::<TiledColliderStrategy>();
        app.update();
        assert_eq!(colliders_area(&app, ground), vec![512.]);
    }

    #[test]
    fn disabled_tile_is_excluded_from_its_collider_chunk() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        spawn_map_with(
            &mut app,
            DYNAMIC_MAP,
            TiledPhysicsSettings::<TestBackend> {
                tiles_chunk_size: TiledColliderChunkSize::Size(UVec2::ONE),
                ..default()
            },
        );
        let ground = layer_by_name(&mut app, "ground");
        let tile = tile_at(&mut app, ground, TilePos::new(1, 3));

        // The chunk of the disabled tile is removed, the other one remains
        app.world_mut()
            .entity_mut(tile)
            .insert(TiledColliderStrategy::Disabled);
        app.update();
        let chunks = collider_chunks(&mut app, ground);
        assert_eq!(
            chunks.iter().map(|(c, _, a)| (*c, *a)).collect::<Vec<_>>(),
            vec![((0, 3), 256.)]
        );

        // Removing the strategy restores its chunk
        app.world_mut()
            .entity_mut(tile)
            .remove::<TiledColliderStrategy>();
        app.update();
        let chunks = collider_chunks(&mut app, ground);
        assert_eq!(
            chunks.iter().map(|(c, _, a)| (*c, *a)).collect::<Vec<_>>(),
            vec![((0, 3), 256.), ((1, 3), 256.)]
        );
    }

    #[test]
    fn collider_chunk_size_can_be_overridden_for_a_layer() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        spawn_map_with(
            &mut app,
            DYNAMIC_MAP,
            TiledPhysicsSettings::<TestBackend> {
                tiles_chunk_size: TiledColliderChunkSize::Size(UVec2::ONE),
                ..default()
            },
        );
        let ground = layer_by_name(&mut app, "ground");
        assert_eq!(collider_chunks(&mut app, ground).len(), 2);

        // Disable chunking for this layer: a single collider, held by the layer itself
        app.world_mut()
            .entity_mut(ground)
            .insert(TiledColliderChunkSize::Disabled);
        app.update();
        assert!(collider_chunks(&mut app, ground).is_empty());
        assert_eq!(colliders_area(&app, ground), vec![512.]);
        assert!(app.world().get::<TestCollider>(ground).is_some());

        // Use larger chunks for this layer: both tiles are in the same chunk
        app.world_mut()
            .entity_mut(ground)
            .insert(TiledColliderChunkSize::Size(UVec2::splat(2)));
        app.update();
        let chunks = collider_chunks(&mut app, ground);
        assert_eq!(
            chunks.iter().map(|(c, _, a)| (*c, *a)).collect::<Vec<_>>(),
            vec![((0, 1), 512.)]
        );
        assert!(app.world().get::<TestCollider>(ground).is_none());

        // Remove the override: back to the map chunk size
        app.world_mut()
            .entity_mut(ground)
            .remove::<TiledColliderChunkSize>();
        app.update();
        assert_eq!(collider_chunks(&mut app, ground).len(), 2);

        // A null chunk size disables chunking
        app.world_mut()
            .entity_mut(ground)
            .insert(TiledColliderChunkSize::Size(UVec2::new(4, 0)));
        app.update();
        assert!(collider_chunks(&mut app, ground).is_empty());
        assert_eq!(colliders_area(&app, ground), vec![512.]);
    }

    #[test]
    fn modifying_settings_updates_colliders_without_respawning_the_map() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        let map = spawn_test_map(&mut app);
        let ground = layer_by_name(&mut app, "ground");
        let rectangle = object_by_name(&mut app, "rectangle");

        // A modification made to the map at runtime
        let tile = tile_at(&mut app, ground, TilePos::new(0, 3));
        app.world_mut().entity_mut(tile).despawn();
        app.update();
        assert_eq!(colliders_area(&app, ground), vec![256.]);

        let mut settings = app
            .world_mut()
            .get_mut::<TiledPhysicsSettings<TestBackend>>(map)
            .unwrap();
        settings.collider_strategy = TiledColliderStrategy::Triangulation;
        settings.tiles_chunk_size = TiledColliderChunkSize::Size(UVec2::ONE);
        app.update();

        // Colliders follow the new settings, layers and objects are the same entities
        // and the runtime modification is kept
        assert_eq!(layer_by_name(&mut app, "ground"), ground);
        assert_eq!(object_by_name(&mut app, "rectangle"), rectangle);
        assert_eq!(collider_chunks(&mut app, ground).len(), 1);
        assert_eq!(
            colliders_strategies(&app, ground),
            vec![TiledColliderStrategy::Triangulation]
        );
        assert_eq!(
            colliders_strategies(&app, rectangle),
            vec![TiledColliderStrategy::Triangulation]
        );

        // Colliders can be disabled for the whole map
        let mut settings = app
            .world_mut()
            .get_mut::<TiledPhysicsSettings<TestBackend>>(map)
            .unwrap();
        settings.collider_strategy = TiledColliderStrategy::Disabled;
        app.update();
        assert!(colliders_area(&app, ground).is_empty());
        assert!(colliders_area(&app, rectangle).is_empty());
        assert_eq!(layer_by_name(&mut app, "ground"), ground);
    }

    #[test]
    fn layers_are_not_chunked_by_default() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        spawn_test_map(&mut app);
        let ground = layer_by_name(&mut app, "ground");
        assert!(collider_chunks(&mut app, ground).is_empty());
        assert_eq!(colliders_area(&app, ground), vec![512.]);

        // But chunking can be enabled for a single layer
        app.world_mut()
            .entity_mut(ground)
            .insert(TiledColliderChunkSize::Size(UVec2::ONE));
        app.update();
        assert_eq!(collider_chunks(&mut app, ground).len(), 2);
    }

    #[test]
    fn additional_colliders_are_children_and_are_despawned() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TestBackend>::default());
        spawn_test_map(&mut app);
        let ground = layer_by_name(&mut app, "ground");
        let colliders_of_ground = |app: &mut App| {
            app.world_mut()
                .query::<(Entity, &TiledColliderOf)>()
                .iter(app.world())
                .filter(|(_, of)| of.0 == ground)
                .map(|(e, _)| e)
                .collect::<Vec<_>>()
        };
        assert_eq!(colliders_of_ground(&mut app), vec![ground]);

        // Without geometry simplification, each of the two tiles gets its own collider:
        // the first one is the layer itself, the other one is a child of the layer
        app.world_mut().entity_mut(ground).insert((
            TiledPhysicsDisableGeometrySimplification,
            TiledColliderStrategy::LineStrip,
        ));
        app.update();
        let colliders = colliders_of_ground(&mut app);
        assert_eq!(colliders.len(), 2);
        assert!(colliders.contains(&ground));
        let child = *colliders.iter().find(|e| **e != ground).unwrap();
        assert_eq!(app.world().get::<ChildOf>(child).unwrap().parent(), ground);
        assert_eq!(colliders_area(&app, ground), vec![256., 256.]);

        // Previous colliders are all removed when colliders are spawned again
        app.world_mut()
            .entity_mut(ground)
            .insert(TiledColliderStrategy::Polyline);
        app.update();
        let colliders = colliders_of_ground(&mut app);
        assert_eq!(colliders.len(), 2);
        assert!(app.world().get_entity(child).is_err());

        // And when the layer does not have any collider anymore
        for pos in [TilePos::new(0, 3), TilePos::new(1, 3)] {
            let tile = tile_at(&mut app, ground, pos);
            app.world_mut().entity_mut(tile).despawn();
        }
        app.update();
        assert!(colliders_of_ground(&mut app).is_empty());
        assert!(app.world().get::<TestCollider>(ground).is_none());
        assert!(app.world().get::<TiledColliderPolygons>(ground).is_none());
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
        app.world_mut()
            .get_mut::<TiledTile>(tile)
            .unwrap()
            .tileset_id = TilesetId(1);
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

/// Tests using a physics backend provided by the crate
#[cfg(feature = "avian")]
mod physics_avian {
    use super::*;
    use avian2d::prelude::Collider;

    #[test]
    fn colliders_are_removed_from_emptied_sources() {
        let mut app = test_app();
        app.add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default());
        spawn_test_map(&mut app);
        let ground = layer_by_name(&mut app, "ground");
        let has_collider = |app: &mut App, entity: Entity| {
            let on_entity = app.world().get::<Collider>(entity).is_some();
            let on_children = app
                .world_mut()
                .query_filtered::<&ChildOf, With<Collider>>()
                .iter(app.world())
                .any(|c| c.parent() == entity);
            on_entity || on_children
        };
        assert!(has_collider(&mut app, ground));

        // Despawn all the tiles of the layer: it should not have any collider anymore
        for pos in [TilePos::new(0, 3), TilePos::new(1, 3)] {
            let tile = tile_at(&mut app, ground, pos);
            app.world_mut().entity_mut(tile).despawn();
        }
        app.update();
        assert!(!has_collider(&mut app, ground));

        // Same for an object whose shape does not produce a collider anymore
        let rectangle = object_by_name(&mut app, "rectangle");
        assert!(has_collider(&mut app, rectangle));
        *app.world_mut().get_mut::<TiledObject>(rectangle).unwrap() = TiledObject::Point;
        app.update();
        assert!(!has_collider(&mut app, rectangle));
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
        app.init_resource::<PropertiesOnMapCreated>();
        app.add_observer(
            |_: On<TiledEvent<MapCreated>>,
             mut found: ResMut<PropertiesOnMapCreated>,
             query: Query<(Option<&Name>, Option<&TilePos>, &TestTileProperty)>| {
                // Tiles only have a name with the `debug` feature: use their position instead
                let mut values = query
                    .iter()
                    .map(|(name, pos, p)| {
                        let label = match (pos, name) {
                            (Some(pos), _) => format!("TiledMapTile({},{})", pos.x, pos.y),
                            (None, Some(name)) => name.to_string(),
                            (None, None) => String::new(),
                        };
                        (label, p.value)
                    })
                    .collect::<Vec<_>>();
                values.sort();
                found.0 = Some(values);
            },
        );
        app
    }

    /// Custom property holding an asset handle, set on an object of the image property test map
    #[derive(Component, Reflect, Default, Debug)]
    #[reflect(Component, Default)]
    pub struct TestImageProperty {
        image: Handle<Image>,
    }

    #[test]
    fn handle_properties_are_loaded_with_their_asset_type() {
        let mut app = properties_test_app();
        app.register_type::<TestImageProperty>();
        spawn_map(&mut app, "tests/properties/image_property.tmx");

        let object = object_by_name(&mut app, "image_object");
        let image = app
            .world()
            .get::<TestImageProperty>(object)
            .expect("missing TestImageProperty")
            .image
            .clone();

        // The handle must point to the image loaded from a path relative to the map, not to
        // a `LoadedUntypedAsset` (which used to be reinterpreted as an unrelated image)
        let path = app
            .world()
            .resource::<AssetServer>()
            .get_path(image.id())
            .unwrap_or_else(|| panic!("image handle has no path: {image:?}"));
        assert!(
            path.path().ends_with("bevy_icon.png"),
            "unexpected path {path}"
        );
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
                TiledTile::new(TilesetId(0), 0),
                ChildOf(ground),
                TilePos::new(2, 2),
            ))
            .id();
        let other_tile = app
            .world_mut()
            .spawn((
                TiledTile::new(TilesetId(0), 1),
                ChildOf(ground),
                TilePos::new(3, 3),
            ))
            .id();
        let tile_object = app
            .world_mut()
            .spawn((
                TiledObject::Tile {
                    width: 16.,
                    height: 16.,
                    tileset_id: TilesetId(0),
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
///
/// Environment variables:
/// - `PERF_RUNS`: number of times each scenario is run
/// - `PERF_CHUNK_SIZE`: size, in tiles, of physics colliders chunks (no chunk by default)
mod perf {
    use super::*;

    const LARGE_MAP: &str = "tests/perf/large_map.tmx";

    /// Number of times each scenario is run, unless overridden by the `PERF_RUNS` environment variable
    const DEFAULT_RUNS: usize = 5;

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

    /// Steps of a frame whose duration is measured separately
    const STEPS: [&str; 4] = ["data", "storage", "render", "physics"];

    /// Time spent in each of [`STEPS`] since the last reset
    #[derive(Resource, Default)]
    struct StepsTimings {
        /// When the current step started
        started: Option<Instant>,
        /// Time spent in each step
        spent: [Duration; STEPS.len()],
    }

    /// Start measuring a step
    fn start_step(mut timings: ResMut<StepsTimings>) {
        timings.started = Some(Instant::now());
    }

    /// Stop measuring step `STEP`
    fn end_step<const STEP: usize>(mut timings: ResMut<StepsTimings>) {
        if let Some(started) = timings.started.take() {
            timings.spent[STEP] += started.elapsed();
        }
    }

    /// Measure the time spent spawning map data, then in each of the `Sync*` system sets.
    ///
    /// Commands are applied by the sync points inserted between system sets: they are accounted
    /// in the step which queued them.
    fn add_steps_timings(app: &mut App) {
        app.init_resource::<StepsTimings>();
        app.add_systems(
            PreUpdate,
            (
                start_step.before(TiledPreUpdateSystems::ProcessLoadedMaps),
                end_step::<0>.after(TiledPreUpdateSystems::ProcessLoadedMaps),
            ),
        );
        app.add_systems(
            PostUpdate,
            (
                start_step
                    .after(TiledPostUpdateSystems::HandleMapAssetEvents)
                    .before(TiledPostUpdateSystems::SyncTiles),
                (end_step::<1>, start_step)
                    .chain()
                    .after(TiledPostUpdateSystems::SyncTiles)
                    .before(TiledPostUpdateSystems::SyncRender),
                (end_step::<2>, start_step)
                    .chain()
                    .after(TiledPostUpdateSystems::SyncRender)
                    .before(TiledPostUpdateSystems::SyncPhysics),
                end_step::<3>
                    .after(TiledPostUpdateSystems::SyncPhysics)
                    .before(TiledPostUpdateSystems::Last),
            ),
        );
    }

    /// Duration of a scenario: whole frames, and time spent in each of [`STEPS`]
    #[derive(Clone, Copy, Default)]
    struct Measure {
        total: Duration,
        steps: [Duration; STEPS.len()],
    }

    /// Run several frames, applying `modify` before each of them, and return the average durations
    fn measure(app: &mut App, frames: u32, mut modify: impl FnMut(&mut App)) -> Measure {
        let mut measure = Measure::default();
        for _ in 0..frames {
            modify(app);
            *app.world_mut().resource_mut::<StepsTimings>() = StepsTimings::default();
            let start = Instant::now();
            app.update();
            measure.total += start.elapsed();
            let spent = app.world().resource::<StepsTimings>().spent;
            for (step, spent) in measure.steps.iter_mut().zip(spent) {
                *step += spent;
            }
        }
        measure.total /= frames;
        for step in &mut measure.steps {
            *step /= frames;
        }
        measure
    }

    fn layer_tiles_entities(app: &mut App, layer: Entity) -> Vec<Entity> {
        app.world_mut()
            .query_filtered::<(Entity, &ChildOf), With<TiledTile>>()
            .iter(app.world())
            .filter(|(_, c)| c.parent() == layer)
            .map(|(e, _)| e)
            .collect()
    }

    /// Run all the scenarios once, on a new app, and return their name and measure
    fn run_scenarios() -> Vec<(String, Measure)> {
        let mut app = test_app();
        #[cfg(feature = "physics")]
        app.add_plugins(TiledPhysicsPlugin::<physics::TestBackend>::default());
        add_steps_timings(&mut app);
        let mut measures = Vec::new();

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
        let map = app.world_mut().spawn(TiledMap(handle)).id();
        // Colliders chunk size can be set using the `PERF_CHUNK_SIZE` environment variable
        #[cfg(feature = "physics")]
        if let Some(chunk_size) = chunk_size() {
            app.world_mut()
                .entity_mut(map)
                .insert(TiledPhysicsSettings::<physics::TestBackend> {
                    tiles_chunk_size: TiledColliderChunkSize::Size(UVec2::splat(chunk_size)),
                    ..default()
                });
        }
        let mut spawn = Measure::default();
        while app
            .world()
            .resource::<Received>()
            .entities_on_map_created
            .is_none()
        {
            let frame = measure(&mut app, 1, |_| {});
            spawn.total += frame.total;
            for (step, spent) in spawn.steps.iter_mut().zip(frame.steps) {
                *step += spent;
            }
        }
        measures.push(("spawn (until MapCreated)".to_string(), spawn));
        measures.push(("first frame".to_string(), measure(&mut app, 1, |_| {})));

        let ground = layer_by_name(&mut app, "ground");
        let decoration = layer_by_name(&mut app, "decoration");
        let mut rng = Lcg(42);

        // Frame without any modification
        measures.push(("idle frame".to_string(), measure(&mut app, 20, |_| {})));

        // Change the tile ID of 100 ground tiles per frame
        let ground_tiles = layer_tiles_entities(&mut app, ground);
        let change_ids = measure(&mut app, 20, |app| {
            for _ in 0..100 {
                let tile = ground_tiles[rng.next(ground_tiles.len() as u32) as usize];
                let mut tile = app.world_mut().get_mut::<TiledTile>(tile).unwrap();
                tile.tile_id = (tile.tile_id + 1) % 4;
            }
        });
        measures.push(("change 100 tile IDs / frame".to_string(), change_ids));

        // Move 100 decoration tiles per frame (some of them replace other tiles)
        let move_tiles = measure(&mut app, 20, |app| {
            let decoration_tiles = layer_tiles_entities(app, decoration);
            for _ in 0..100 {
                let tile = decoration_tiles[rng.next(decoration_tiles.len() as u32) as usize];
                if let Some(mut pos) = app.world_mut().get_mut::<TilePos>(tile) {
                    *pos = TilePos::new(rng.next(200), rng.next(200));
                }
            }
        });
        measures.push(("move 100 tiles / frame".to_string(), move_tiles));

        // Despawn 1000 ground tiles in a single frame
        let ground_tiles = layer_tiles_entities(&mut app, ground);
        let despawn = measure(&mut app, 1, |app| {
            for &tile in ground_tiles.iter().take(1000) {
                app.world_mut().entity_mut(tile).despawn();
            }
        });
        measures.push(("despawn 1000 tiles".to_string(), despawn));

        // Spawn many objects in a new objects layer, then despawn this layer:
        // each object is removed from the map storage one by one
        const OBJECTS: u32 = 5000;
        let objects_layer = app
            .world_mut()
            .spawn((TiledLayer::Objects, TiledMapReference(map), ChildOf(map)))
            .id();
        let spawn_objects = measure(&mut app, 1, |app| {
            for i in 0..OBJECTS {
                app.world_mut().spawn((
                    TiledObject::Rectangle {
                        width: 4.,
                        height: 4.,
                    },
                    Transform::from_xyz((i % 200) as f32 * 16., (i / 200) as f32 * -16., 0.),
                    ChildOf(objects_layer),
                ));
            }
        });
        measures.push((format!("spawn {OBJECTS} objects"), spawn_objects));
        let despawn_objects = measure(&mut app, 1, |app| {
            app.world_mut().entity_mut(objects_layer).despawn();
        });
        measures.push((format!("despawn {OBJECTS} objects layer"), despawn_objects));
        assert!(map_entities::<With<TiledObject>>(&mut app, map).is_empty());

        // Despawn the whole map
        let despawn_map = measure(&mut app, 1, |app| {
            app.world_mut().entity_mut(map).despawn();
        });
        measures.push(("despawn map".to_string(), despawn_map));

        measures
    }

    /// Size of colliders chunks, if the `PERF_CHUNK_SIZE` environment variable is set
    fn chunk_size() -> Option<u32> {
        std::env::var("PERF_CHUNK_SIZE")
            .ok()
            .and_then(|size| size.parse().ok())
    }

    fn median(mut values: Vec<Duration>) -> Duration {
        values.sort();
        values[values.len() / 2]
    }

    fn ms(duration: Duration) -> String {
        format!("{:.1}", duration.as_secs_f64() * 1000.)
    }

    #[test]
    #[ignore = "performance test: cargo test --release --all-features --lib perf -- --ignored --nocapture"]
    fn perf_large_map() {
        let runs = std::env::var("PERF_RUNS")
            .ok()
            .and_then(|runs| runs.parse().ok())
            .unwrap_or(DEFAULT_RUNS)
            .max(1);
        let all = (0..runs).map(|_| run_scenarios()).collect::<Vec<_>>();

        eprintln!(
            "=== Performance on a 200x200 map with 2 tiles layers (44137 tiles), {runs} runs, colliders chunk size: {:?}",
            chunk_size()
        );
        eprintln!(
            "{:<32} {:>8} {:>8} | {:>8} {:>8} {:>8} {:>8}",
            "scenario (ms, median)", "total", "min", STEPS[0], STEPS[1], STEPS[2], STEPS[3]
        );
        for (index, (name, _)) in all[0].iter().enumerate() {
            let measures = all.iter().map(|run| run[index].1).collect::<Vec<_>>();
            let totals = measures.iter().map(|m| m.total).collect::<Vec<_>>();
            let min = *totals.iter().min().unwrap();
            let steps = (0..STEPS.len())
                .map(|step| ms(median(measures.iter().map(|m| m.steps[step]).collect())))
                .collect::<Vec<_>>();
            eprintln!(
                "{:<32} {:>8} {:>8} | {:>8} {:>8} {:>8} {:>8}",
                name,
                ms(median(totals)),
                ms(min),
                steps[0],
                steps[1],
                steps[2],
                steps[3]
            );
        }
    }
}
