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
    let handle = app
        .world()
        .resource::<AssetServer>()
        .load::<TiledMapAsset>(DYNAMIC_MAP);
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

#[cfg(any(feature = "render", feature = "physics"))]
fn layer_by_name(app: &mut App, name: &str) -> Entity {
    app.world_mut()
        .query_filtered::<(Entity, &TiledName), With<TiledLayer>>()
        .iter(app.world())
        .find(|(_, n)| n.0 == name)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("missing layer {name}"))
}

#[cfg(any(feature = "render", feature = "physics"))]
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
}

#[cfg(feature = "physics")]
mod physics {
    use super::*;
    use ::geo::Area;

    /// Physics backend which only spawns an entity holding the collider polygons
    #[derive(Default, Clone, Debug, Reflect)]
    struct TestBackend;

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
