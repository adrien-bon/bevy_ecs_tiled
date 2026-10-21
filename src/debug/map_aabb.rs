//! Debug plugin for visualizing Tiled maps bounding boxes.
//!
//! This module provides a plugin and configuration for drawing the axis-aligned bounding box (AABB)
//! of each Tiled map using Bevy [`Gizmos`]. It is useful for debugging map placement, anchoring
//! and coordinate conversions.
//!
//! When enabled, the plugin draws two `rect_2d` gizmos for each map:
//!   - the AABB of the tiles grid, as computed by `bevy_ecs_tilemap` helpers (ie. where tiles are rendered);
//!   - the AABB obtained by converting the map corners from Tiled coordinates using
//!     [`TiledMapAsset::world_space_from_tiled_position`].
//!
//! Both rectangles should overlap: any mismatch hints at an issue in coordinates conversion.

use crate::prelude::*;
use bevy::{
    color::palettes::css::{AQUA, ORANGE},
    prelude::*,
};

/// Configuration for the [`TiledDebugMapAabbPlugin`].
///
/// Allows customization of the appearance of the `rect_2d` [`Gizmos`] drawn for each map.
#[derive(Resource, Reflect, Copy, Clone, Debug)]
#[reflect(Resource, Debug)]
pub struct TiledDebugMapAabbConfig {
    /// [`Color`] of the `rect_2d` [`Gizmos`] for the map tiles grid AABB as computed by `bevy_ecs_tilemap`.
    ///
    /// If [`None`], this AABB will not be displayed.
    pub tilemap_aabb_color: Option<Color>,
    /// [`Color`] of the `rect_2d` [`Gizmos`] for the map AABB as computed using
    /// [`TiledMapAsset::world_space_from_tiled_position`].
    ///
    /// If [`None`], this AABB will not be displayed.
    pub tiled_aabb_color: Option<Color>,
}

impl Default for TiledDebugMapAabbConfig {
    fn default() -> Self {
        Self {
            tilemap_aabb_color: Some(Color::from(ORANGE)),
            tiled_aabb_color: Some(Color::from(AQUA)),
        }
    }
}

/// Debug [`Plugin`] for visualizing Tiled maps bounding boxes.
///
/// Add this plugin to your app to display, for each [`TiledMap`], two colored rectangles:
/// - the AABB of the map tiles grid as computed by `bevy_ecs_tilemap` (ie. where tiles are rendered),
/// - the map AABB obtained by converting the map corners from Tiled coordinates to world space.
///
/// If both rectangles do not overlap, there is probably an issue with coordinates conversion.
///
/// Note that for infinite maps, the AABB only covers the chunks containing tiles: objects
/// located outside of these chunks will lie outside of the AABB.
///
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledDebugMapAabbPlugin::default());
/// ```
#[derive(Default, Copy, Clone, Debug)]
pub struct TiledDebugMapAabbPlugin(pub TiledDebugMapAabbConfig);

impl Plugin for TiledDebugMapAabbPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<TiledDebugMapAabbConfig>()
            .insert_resource(self.0)
            .add_systems(Update, draw_maps_aabb);
    }
}

fn draw_maps_aabb(
    map_query: Query<(&TiledMap, &GlobalTransform, &TilemapAnchor)>,
    assets: Res<Assets<TiledMapAsset>>,
    config: Res<TiledDebugMapAabbConfig>,
    mut gizmos: Gizmos,
) {
    if config.tilemap_aabb_color.is_none() && config.tiled_aabb_color.is_none() {
        return;
    }
    for (tiled_map, transform, anchor) in map_query.iter() {
        let Some(map_asset) = assets.get(&tiled_map.0) else {
            continue;
        };
        if let Some(color) = config.tilemap_aabb_color {
            draw_aabb(
                &mut gizmos,
                transform,
                tilemap_corners(map_asset, anchor),
                color,
            );
        }
        if let Some(color) = config.tiled_aabb_color {
            draw_aabb(
                &mut gizmos,
                transform,
                tiled_corners(map_asset, anchor),
                color,
            );
        }
    }
}

/// Map grid corners, relative to the map entity, as computed by `bevy_ecs_tilemap`.
fn tilemap_corners(map_asset: &TiledMapAsset, anchor: &TilemapAnchor) -> [Vec2; 4] {
    let map_type = tilemap_type_from_map(&map_asset.map);
    let grid_size = grid_size_from_map(&map_asset.map);
    let tile_size = map_asset.largest_tile_size;
    let map_size = map_asset.tilemap_size;
    // Only consider the grid footprint: tiles images bigger than the grid would otherwise
    // enlarge the AABB and prevent comparing it with the one computed from Tiled coordinates.
    let aabb = chunk_aabb(
        UVec2::new(map_size.x - 1, map_size.y - 1),
        &grid_size,
        &TilemapTileSize::new(grid_size.x, grid_size.y),
        &map_type,
    );
    // Anchor offset still depends on the actual tile size
    let offset = anchor.as_offset(&map_size, &grid_size, &tile_size, &map_type);
    let min = Vec2::new(aabb.min().x, aabb.min().y) + offset;
    let max = Vec2::new(aabb.max().x, aabb.max().y) + offset;
    [
        Vec2::new(min.x, min.y),
        Vec2::new(min.x, max.y),
        Vec2::new(max.x, max.y),
        Vec2::new(max.x, min.y),
    ]
}

/// Map corners, relative to the map entity, as computed by converting
/// Tiled coordinates using [`TiledMapAsset::world_space_from_tiled_position`].
fn tiled_corners(map_asset: &TiledMapAsset, anchor: &TilemapAnchor) -> [Vec2; 4] {
    let map_type = tilemap_type_from_map(&map_asset.map);
    let grid_size = grid_size_from_map(&map_asset.map);
    let map_size = map_asset.tilemap_size;

    // Size of a Tiled "unit" along each axis, used to compute top-left chunk origin.
    // Must be consistent with how `tiled_offset` is computed by the map loader.
    let (unit, extent) = match map_type {
        TilemapType::Isometric(_) => (
            Vec2::splat(grid_size.y),
            Vec2::new(map_size.x as f32, map_size.y as f32) * grid_size.y,
        ),
        TilemapType::Hexagon(HexCoordSystem::ColumnOdd | HexCoordSystem::ColumnEven) => (
            Vec2::new(grid_size.x * 0.75, grid_size.y),
            map_asset.rect.size(),
        ),
        TilemapType::Hexagon(_) => (
            Vec2::new(grid_size.x, grid_size.y * 0.75),
            map_asset.rect.size(),
        ),
        _ => (Vec2::new(grid_size.x, grid_size.y), map_asset.rect.size()),
    };

    // For finite maps, top-left chunk is always (0, 0)
    let origin = Vec2::new(
        map_asset.topleft_chunk.0 as f32 * tiled::ChunkData::WIDTH as f32,
        map_asset.topleft_chunk.1 as f32 * tiled::ChunkData::HEIGHT as f32,
    ) * unit;

    [
        origin,
        origin + Vec2::new(extent.x, 0.),
        origin + extent,
        origin + Vec2::new(0., extent.y),
    ]
    .map(|corner| map_asset.world_space_from_tiled_position(anchor, corner))
}

fn draw_aabb(gizmos: &mut Gizmos, transform: &GlobalTransform, corners: [Vec2; 4], color: Color) {
    let corners = corners.map(|c| transform.transform_point(c.extend(0.)).truncate());
    let aabb = Aabb2d::from_point_cloud(Isometry2d::IDENTITY, &corners);
    gizmos.rect_2d(
        Isometry2d::from_translation(aabb.center()),
        aabb.half_size() * 2.,
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::LoadState;
    use std::time::{Duration, Instant};

    const ANCHORS: [TilemapAnchor; 11] = [
        TilemapAnchor::None,
        TilemapAnchor::TopLeft,
        TilemapAnchor::TopRight,
        TilemapAnchor::TopCenter,
        TilemapAnchor::CenterRight,
        TilemapAnchor::CenterLeft,
        TilemapAnchor::BottomLeft,
        TilemapAnchor::BottomRight,
        TilemapAnchor::BottomCenter,
        TilemapAnchor::Center,
        TilemapAnchor::Custom(Vec2::new(0.25, -0.1)),
    ];

    /// Load all maps from the given `assets/maps` sub-directories.
    fn load_maps(dirs: &[&str]) -> (App, Vec<(String, Handle<TiledMapAsset>)>) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Image>();
        app.init_asset::<TextureAtlasLayout>();
        app.add_plugins((
            crate::tiled::cache::plugin,
            crate::tiled::map::asset::plugin,
            crate::tiled::map::loader::plugin,
        ));

        let mut maps = vec![];
        for dir in dirs {
            for entry in std::fs::read_dir(format!("assets/maps/{dir}")).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().is_none_or(|ext| ext != "tmx") {
                    continue;
                }
                let path = format!("maps/{dir}/{}", path.file_name().unwrap().to_string_lossy());
                let handle: Handle<TiledMapAsset> =
                    app.world().resource::<AssetServer>().load(&path);
                let deadline = Instant::now() + Duration::from_secs(10);
                loop {
                    assert!(
                        Instant::now() < deadline,
                        "{path}: map did not load in time"
                    );
                    app.update();
                    match app.world().resource::<AssetServer>().load_state(&handle) {
                        LoadState::Loaded => break,
                        LoadState::Failed(error) => panic!("{path}: failed to load: {error}"),
                        _ => std::thread::sleep(Duration::from_millis(1)),
                    }
                }
                maps.push((path, handle));
            }
        }
        assert!(!maps.is_empty());
        (app, maps)
    }

    fn aabb(corners: [Vec2; 4]) -> Aabb2d {
        Aabb2d::from_point_cloud(Isometry2d::IDENTITY, &corners)
    }

    /// Map AABB computed from Tiled coordinates conversion must match the one
    /// computed by `bevy_ecs_tilemap`, whatever the map orientation and anchor.
    #[test]
    fn tiled_aabb_matches_tilemap_aabb() {
        let (app, maps) = load_maps(&["orthogonal", "isometric", "hexagonal"]);
        let assets = app.world().resource::<Assets<TiledMapAsset>>();
        for (path, handle) in maps {
            let map_asset = assets.get(&handle).unwrap();
            for anchor in ANCHORS {
                let tilemap = aabb(tilemap_corners(map_asset, &anchor));
                let tiled = aabb(tiled_corners(map_asset, &anchor));
                assert!(
                    tilemap.min.abs_diff_eq(tiled.min, 0.01)
                        && tilemap.max.abs_diff_eq(tiled.max, 0.01),
                    "{path} ({anchor:?}): tilemap AABB {tilemap:?} != tiled AABB {tiled:?}"
                );
            }
        }
    }

    /// For isometric maps, Tiled "screen" origin (used to position image layers) is located
    /// `map.height * tile_width / 2` pixels on the left of tile `(0, 0)` top corner.
    ///
    /// Expected position is computed from `bevy_ecs_tilemap` tiles positions.
    #[test]
    fn isometric_screen_origin() {
        let (app, maps) = load_maps(&["isometric"]);
        let assets = app.world().resource::<Assets<TiledMapAsset>>();
        for (path, handle) in maps {
            let map_asset = assets.get(&handle).unwrap();
            let grid_size = grid_size_from_map(&map_asset.map);
            let map_size = map_asset.tilemap_size;
            // Top-left tile in bevy_ecs_tilemap coordinates (Y axis is flipped)
            // and its coordinates in Tiled (offset by the top-left chunk for infinite maps)
            let tile_pos = TilePos::new(0, map_size.y - 1);
            let tiled_tile = Vec2::new(
                (map_asset.topleft_chunk.0 * tiled::ChunkData::WIDTH as i32) as f32,
                (map_asset.topleft_chunk.1 * tiled::ChunkData::HEIGHT as i32) as f32,
            );
            // Tiled screen position of this tile top corner, relative to Tiled screen origin
            let tiled_screen = Vec2::new(
                (tiled_tile.x - tiled_tile.y + map_asset.map.height as f32) * grid_size.x / 2.,
                (tiled_tile.x + tiled_tile.y) * grid_size.y / 2.,
            );
            for anchor in ANCHORS {
                let top_corner = tile_pos.center_in_world(
                    &map_size,
                    &grid_size,
                    &map_asset.largest_tile_size,
                    &tilemap_type_from_map(&map_asset.map),
                    &anchor,
                ) + Vec2::new(0., grid_size.y / 2.);
                // Tiled Y axis points down
                let expected = top_corner + Vec2::new(-tiled_screen.x, tiled_screen.y);
                let origin = map_asset.screen_origin(&anchor);
                assert!(
                    origin.abs_diff_eq(expected, 0.01),
                    "{path} ({anchor:?}): screen origin {origin:?} != expected {expected:?}"
                );
            }
        }
    }
}
