//! Mouse picking for tiles.
//!
//! Tiles are not regular sprites: they are rendered by their tilemap and do not even have
//! a `Transform`, so Bevy does not know how to pick them. We provide our own picking backend,
//! which tells Bevy which tiles are below the pointer: tiles entities then receive the usual
//! pointer events (`PointerClick`, `PointerOver`, ...) and we can observe them as we would do
//! for any other entity.

use bevy::{
    ecs::system::SystemParam,
    picking::{
        PickingSystems,
        backend::{HitData, PointerHits, ray::RayMap},
    },
    prelude::*,
};
use bevy_ecs_tiled::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PreUpdate,
        tile_picking_backend.in_set(PickingSystems::Backend),
    );
}

/// What we need to know about a tilemap to convert between world and tiles positions.
type TilemapGeometry = (
    &'static GlobalTransform,
    &'static TilemapSize,
    &'static TilemapGridSize,
    &'static TilemapTileSize,
    &'static TilemapType,
    &'static TilemapAnchor,
);

/// Returns the position of the tile at the provided world position, if it is within the tilemap.
fn tile_at(
    (transform, size, grid_size, tile_size, map_type, anchor): (
        &GlobalTransform,
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TilemapAnchor,
    ),
    world_position: Vec2,
) -> Option<TilePos> {
    // Tiles positions are relative to their tilemap
    let tilemap_position = transform
        .affine()
        .inverse()
        .transform_point3(world_position.extend(0.))
        .truncate();
    TilePos::from_world_pos(
        &tilemap_position,
        size,
        grid_size,
        tile_size,
        map_type,
        anchor,
    )
}

/// Conversion between world and tiles positions, for the map currently loaded.
///
/// All the tilemaps of a map share the same geometry, so we use the first one we find.
#[derive(SystemParam)]
pub struct TileGeometry<'w, 's> {
    tilemaps_query: Query<'w, 's, TilemapGeometry, With<TiledTilemap>>,
}

impl TileGeometry<'_, '_> {
    /// Returns the position of the tile at the provided world position, if it is within the map.
    pub fn tile_at(&self, world_position: Vec2) -> Option<TilePos> {
        tile_at(self.tilemaps_query.iter().next()?, world_position)
    }

    /// Returns the world position of the center of a tile.
    pub fn tile_center(&self, position: TilePos) -> Option<Vec2> {
        let (transform, size, grid_size, tile_size, map_type, anchor) =
            self.tilemaps_query.iter().next()?;
        let tilemap_position =
            position.center_in_world(size, grid_size, tile_size, map_type, anchor);
        Some(
            transform
                .transform_point(tilemap_position.extend(0.))
                .truncate(),
        )
    }
}

/// Report the tiles below each pointer to Bevy picking.
///
/// There is one tilemap for each layer and tileset: we report the tile of each of them, along
/// with its depth. Bevy then sends pointer events to the topmost one.
fn tile_picking_backend(
    ray_map: Res<RayMap>,
    cameras_query: Query<&Camera>,
    tilemaps_query: Query<(TilemapGeometry, &TileStorage), With<TiledTilemap>>,
    mut pointer_hits: MessageWriter<PointerHits>,
) {
    for (ray_id, ray) in ray_map.iter() {
        let Ok(camera) = cameras_query.get(ray_id.camera) else {
            continue;
        };

        // With a 2D camera, the ray goes straight through the screen
        let pointer_position = ray.origin.truncate();
        let picks = tilemaps_query
            .iter()
            .filter_map(|(geometry, storage)| {
                let tile_position = tile_at(geometry, pointer_position)?;
                let tile = storage.get(&tile_position)?;
                let tilemap_z = geometry.0.translation().z;
                Some((
                    tile,
                    HitData::new(
                        ray_id.camera,
                        ray.origin.z - tilemap_z,
                        Some(pointer_position.extend(tilemap_z)),
                        None,
                    ),
                ))
            })
            .collect::<Vec<_>>();

        if !picks.is_empty() {
            pointer_hits.write(PointerHits::new(ray_id.pointer, picks, camera.order as f32));
        }
    }
}
