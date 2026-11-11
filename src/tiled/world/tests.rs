//! Tests checking that worlds are correctly spawned, whatever the orientation of their maps.

use crate::prelude::*;
use bevy::{asset::LoadState, prelude::*, transform::TransformPlugin};
use std::time::{Duration, Instant};

/// Worlds to test, covering all the supported map orientations
const WORLDS: [&str; 4] = [
    "worlds/orthogonal.world",
    "worlds/isometric.world",
    "worlds/hexagonal_pointy_top.world",
    "worlds/hexagonal_flat_top.world",
];

/// World anchors to test, with the expected offset of the world top-left corner
/// for a world of the given size.
fn anchors(world_size: Vec2) -> [(TilemapAnchor, Vec2); 4] {
    [
        (TilemapAnchor::None, Vec2::new(0., world_size.y)),
        (TilemapAnchor::TopLeft, Vec2::ZERO),
        (
            TilemapAnchor::Center,
            Vec2::new(-world_size.x / 2., world_size.y / 2.),
        ),
        (
            TilemapAnchor::BottomRight,
            Vec2::new(-world_size.x, world_size.y),
        ),
    ]
}

/// Events received from the world
#[derive(Resource, Default)]
struct WorldEvents {
    worlds_created: Vec<Entity>,
    maps_created: Vec<Entity>,
}

fn world_app() -> App {
    let mut app = crate::tests::test_app();
    app.add_plugins(TransformPlugin);
    app.init_resource::<WorldEvents>();
    app.add_observer(
        |ev: On<TiledEvent<WorldCreated>>, mut events: ResMut<WorldEvents>| {
            let ev = ev.event();
            if ev.entity == ev.origin {
                events.worlds_created.push(ev.origin);
            }
        },
    );
    app.add_observer(
        |ev: On<TiledEvent<MapCreated>>, mut events: ResMut<WorldEvents>| {
            // Events are propagated to parent entities: only count the original one
            let ev = ev.event();
            if ev.entity == ev.origin {
                events.maps_created.push(ev.origin);
            }
        },
    );
    app
}

/// Load the given world and wait until it and all its maps are loaded
pub(super) fn load_world(app: &mut App, path: &str) -> Handle<TiledWorldAsset> {
    let handle: Handle<TiledWorldAsset> =
        app.world().resource::<AssetServer>().load(path.to_owned());
    let deadline = Instant::now() + Duration::from_secs(10);
    while !app
        .world()
        .resource::<AssetServer>()
        .is_loaded_with_dependencies(&handle)
    {
        assert!(
            Instant::now() < deadline,
            "{path}: world did not load in time"
        );
        if let LoadState::Failed(error) = app.world().resource::<AssetServer>().load_state(&handle)
        {
            panic!("{path}: failed to load: {error}");
        }
        app.update();
        std::thread::sleep(Duration::from_millis(1));
    }
    handle
}

/// Run the app until all maps of the given world are spawned (no chunking)
fn spawn_world(app: &mut App, path: &str, anchor: TilemapAnchor) -> Entity {
    let handle = load_world(app, path);
    let maps_count = app
        .world()
        .resource::<Assets<TiledWorldAsset>>()
        .get(&handle)
        .unwrap()
        .maps
        .len();
    let world_entity = app.world_mut().spawn((TiledWorld(handle), anchor)).id();
    let deadline = Instant::now() + Duration::from_secs(10);
    while app.world().resource::<WorldEvents>().maps_created.len() < maps_count {
        assert!(
            Instant::now() < deadline,
            "{path}: maps did not spawn in time"
        );
        app.update();
        std::thread::sleep(Duration::from_millis(1));
    }
    // Make sure transforms are propagated
    app.update();
    world_entity
}

/// Raw Tiled world data: world size and the rect of each map, using Tiled coordinates
/// (origin at the top-left corner of the world, Y axis pointing down)
fn tiled_world_layout(world: &tiled::World) -> (Vec2, Vec<Rect>) {
    let rects: Vec<Rect> = world
        .maps
        .iter()
        .map(|m| {
            Rect::new(
                m.x as f32,
                m.y as f32,
                (m.x + m.width.unwrap()) as f32,
                (m.y + m.height.unwrap()) as f32,
            )
        })
        .collect();
    let size = rects.iter().fold(Vec2::ZERO, |acc, r| acc.max(r.max));
    (size, rects)
}

/// Grid of a spawned map, used to compute its tiles position in world space
/// as `bevy_ecs_tilemap` would render them.
struct MapGrid {
    map_size: TilemapSize,
    grid_size: TilemapGridSize,
    tile_size: TilemapTileSize,
    map_type: TilemapType,
    anchor: TilemapAnchor,
    transform: GlobalTransform,
}

impl MapGrid {
    fn new(app: &App, map_entity: Entity) -> Self {
        let world = app.world();
        let map_handle = world.get::<TiledMap>(map_entity).unwrap();
        let map_asset = world
            .resource::<Assets<TiledMapAsset>>()
            .get(&map_handle.0)
            .unwrap();
        Self {
            map_size: map_asset.tilemap_size,
            grid_size: grid_size_from_map(&map_asset.map),
            tile_size: map_asset.largest_tile_size,
            map_type: tilemap_type_from_map(&map_asset.map),
            anchor: *world.get::<TilemapAnchor>(map_entity).unwrap(),
            transform: *world.get::<GlobalTransform>(map_entity).unwrap(),
        }
    }

    /// World space center of the tile at the given Tiled position (Y axis pointing down)
    fn tile_center(&self, x: u32, y: u32) -> Vec2 {
        // `bevy_ecs_tilemap` Y axis is flipped compared to Tiled
        let center = TilePos::new(x, self.map_size.y - 1 - y).center_in_world(
            &self.map_size,
            &self.grid_size,
            &self.tile_size,
            &self.map_type,
            &self.anchor,
        );
        self.transform.transform_point(center.extend(0.)).truncate()
    }

    /// Actual bounding box of the map grid, in world space
    fn aabb(&self) -> Rect {
        let half_tile = Vec2::new(self.grid_size.x, self.grid_size.y) / 2.;
        let mut aabb: Option<Rect> = None;
        for x in 0..self.map_size.x {
            for y in 0..self.map_size.y {
                let center = self.tile_center(x, y);
                let tile = Rect::from_corners(center - half_tile, center + half_tile);
                aabb = Some(aabb.map_or(tile, |aabb| aabb.union(tile)));
            }
        }
        aabb.unwrap()
    }
}

/// Extrapolate the position of the tile just after the last one of a row or column of a grid,
/// from the positions of the last three tiles.
///
/// Works for grids where the tile position is periodic with a period of 1 (orthogonal, isometric)
/// or 2 (hexagonal) tiles: `c(n) = c(n - 2) + (c(n - 1) - c(n - 3))`.
fn next_tile_center(center: impl Fn(u32) -> Vec2, n: u32) -> Vec2 {
    center(n - 2) + center(n - 1) - center(n - 3)
}

/// Without chunking, all maps of the world are spawned as children of the world entity
/// and all the corresponding events are sent.
#[test]
fn world_spawns_all_maps() {
    for path in WORLDS {
        let mut app = world_app();
        let world_entity = spawn_world(&mut app, path, TilemapAnchor::Center);

        let world = app.world();
        let handle = world.get::<TiledWorld>(world_entity).unwrap();
        let maps_count = world
            .resource::<Assets<TiledWorldAsset>>()
            .get(&handle.0)
            .unwrap()
            .maps
            .len();
        let storage = world.get::<TiledWorldStorage>(world_entity).unwrap();
        assert_eq!(storage.maps().count(), maps_count, "{path}");
        for idx in 0..maps_count {
            let map_entity = storage
                .map_entity(WorldMapId(idx as u32))
                .unwrap_or_else(|| panic!("{path}: map {idx} is not spawned"));
            assert_eq!(
                world.get::<ChildOf>(map_entity).unwrap().parent(),
                world_entity,
                "{path}: map {idx} is not a child of the world"
            );
        }

        let events = world.resource::<WorldEvents>();
        assert_eq!(events.worlds_created, vec![world_entity], "{path}");
        assert_eq!(events.maps_created.len(), maps_count, "{path}");
        for map_entity in &events.maps_created {
            assert!(storage.map_id(*map_entity).is_some(), "{path}");
        }
    }
}

/// Each map of the world must cover exactly the area defined in the `.world` file:
/// maps are placed according to their Tiled position, adjacent maps touch each other,
/// whatever the map orientation and world anchor.
#[test]
fn maps_match_world_layout() {
    let mut errors = vec![];
    for path in WORLDS {
        let mut app = world_app();
        let handle = load_world(&mut app, path);
        let (world_size, tiled_rects) = tiled_world_layout(
            &app.world()
                .resource::<Assets<TiledWorldAsset>>()
                .get(&handle)
                .unwrap()
                .world,
        );

        for (anchor, top_left) in anchors(world_size) {
            let mut app = world_app();
            let world_entity = spawn_world(&mut app, path, anchor);
            let storage = app.world().get::<TiledWorldStorage>(world_entity).unwrap();
            for (idx, tiled_rect) in tiled_rects.iter().enumerate() {
                let map_entity = storage.map_entity(WorldMapId(idx as u32)).unwrap();
                // Tiled Y axis points down
                let expected = Rect::from_corners(
                    top_left + Vec2::new(tiled_rect.min.x, -tiled_rect.min.y),
                    top_left + Vec2::new(tiled_rect.max.x, -tiled_rect.max.y),
                );
                let actual = MapGrid::new(&app, map_entity).aabb();
                if !actual.min.abs_diff_eq(expected.min, 0.01)
                    || !actual.max.abs_diff_eq(expected.max, 0.01)
                {
                    errors.push(format!(
                        "{path} ({anchor:?}) map {idx}: actual {actual:?} != expected {expected:?}"
                    ));
                }
            }
        }
    }
    assert!(errors.is_empty(), "\n{}", errors.join("\n"));
}

/// With chunking, only the maps around the camera are spawned
#[test]
fn chunking_spawns_visible_maps() {
    for path in WORLDS {
        let mut app = world_app();
        let handle = load_world(&mut app, path);
        let (world_size, tiled_rects) = tiled_world_layout(
            &app.world()
                .resource::<Assets<TiledWorldAsset>>()
                .get(&handle)
                .unwrap()
                .world,
        );
        // Tiled position of the center of a map
        let map_center = |idx: usize| tiled_rects[idx].center();
        assert!(world_size.x > 0. && world_size.y > 0.);

        let camera = app
            .world_mut()
            .spawn((Camera::default(), Transform::default()))
            .id();
        let world_entity = app
            .world_mut()
            .spawn((
                TiledWorld(handle),
                TilemapAnchor::TopLeft,
                TiledWorldChunking::new(1., 1.),
            ))
            .id();

        for idx in [1, tiled_rects.len() - 1, 0] {
            // Since maps rects can overlap (for instance with isometric maps),
            // camera can see several maps
            let camera_position = map_center(idx);
            let expected: Vec<WorldMapId> = tiled_rects
                .iter()
                .enumerate()
                .filter(|(_, rect)| rect.contains(camera_position))
                .map(|(idx, _)| WorldMapId(idx as u32))
                .collect();
            assert!(expected.contains(&WorldMapId(idx as u32)));

            // Using a TopLeft world anchor: Bevy position is the Tiled one with Y axis flipped
            app.world_mut()
                .get_mut::<Transform>(camera)
                .unwrap()
                .translation = Vec3::new(camera_position.x, -camera_position.y, 0.);
            for _ in 0..3 {
                app.update();
            }
            let storage = app.world().get::<TiledWorldStorage>(world_entity).unwrap();
            let mut spawned: Vec<WorldMapId> = storage.maps().map(|(id, _)| id).collect();
            spawned.sort();
            assert_eq!(spawned, expected, "{path}: camera centered on map {idx}");
        }
    }
}

/// Grids of adjacent maps must be continuous: the tiles of a map exactly continue the grid
/// of its neighbours, as if the world was a single big map.
#[test]
fn adjacent_maps_grids_are_continuous() {
    // World path and the number of maps per row: maps are listed in row-major order
    // in the `.world` file, in tiles space.
    let worlds = [
        ("worlds/orthogonal.world", 5),
        ("worlds/isometric.world", 2),
        ("worlds/hexagonal_pointy_top.world", 2),
        ("worlds/hexagonal_flat_top.world", 2),
    ];
    let mut errors = vec![];
    for (path, columns) in worlds {
        let mut app = world_app();
        let world_entity = spawn_world(&mut app, path, TilemapAnchor::Center);
        let storage = app.world().get::<TiledWorldStorage>(world_entity).unwrap();
        let grids: Vec<MapGrid> = (0..storage.maps().count())
            .map(|idx| MapGrid::new(&app, storage.map_entity(WorldMapId(idx as u32)).unwrap()))
            .collect();

        let mut check = |a: usize, b: usize, expected: Vec2, actual: Vec2, tile: (u32, u32)| {
            if !actual.abs_diff_eq(expected, 0.01) {
                errors.push(format!(
                    "{path}: map {b} tile {tile:?} is at {actual}, expected {expected} to continue map {a} grid"
                ));
            }
        };
        for (a, grid_a) in grids.iter().enumerate() {
            let size = grid_a.map_size;
            // Right neighbour: its first column continues our last one
            if (a + 1) % columns != 0
                && let Some(grid_b) = grids.get(a + 1)
            {
                for y in 0..size.y {
                    let expected = next_tile_center(|x| grid_a.tile_center(x, y), size.x);
                    check(a, a + 1, expected, grid_b.tile_center(0, y), (0, y));
                }
            }
            // Bottom neighbour: its first row continues our last one
            if let Some(grid_b) = grids.get(a + columns) {
                for x in 0..size.x {
                    let expected = next_tile_center(|y| grid_a.tile_center(x, y), size.y);
                    check(a, a + columns, expected, grid_b.tile_center(x, 0), (x, 0));
                }
            }
        }
    }
    assert!(errors.is_empty(), "\n{}", errors.join("\n"));
}
