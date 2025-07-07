//! Chunk management for Tiled worlds.
//!
//! This module implements logic spawning and despawning Tiled maps based on camera position
//! and chunking configuration. It allows for efficient rendering and memory management by only
//! keeping visible maps in memory, while removing those that are not currently in view.

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
};
use bevy_ecs_tilemap::{map::TilemapRenderSettings, prelude::TilemapAnchor};

use super::{asset::TiledWorldAsset, storage::TiledWorldStorage, TiledWorld};
use crate::tiled::{
    map::{TiledMap, TiledMapLayerZOffset},
    sets::TiledPostUpdateSystems,
};

/// [`Component`] holding Tiled world chunking configuration.
///
/// If this value is None, we won't perform chunking: all maps from this world will just be loaded
/// If this value is set, defines the area (in pixel) around each [`Camera`] where we should spawn a
/// map if it overlaps with its associated [`Rect`].
///
/// Must be added to the [`Entity`] holding the world.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledWorldChunking(pub Option<Vec2>);

impl TiledWorldChunking {
    /// Initialize world chunking with provided size
    pub fn new(width: f32, height: f32) -> Self {
        Self(Some(Vec2::new(width, height)))
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledWorldChunking>();
    app.add_systems(
        PostUpdate,
        handle_world_chunking.in_set(TiledPostUpdateSystems::HandleWorldChunking),
    );
}

fn handle_world_chunking(
    camera_query: Query<&Transform, (With<Camera>, Changed<Transform>)>,
    worlds: Res<Assets<TiledWorldAsset>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut world_query: Query<(
        Entity,
        &TiledWorld,
        &GlobalTransform,
        &TiledWorldChunking,
        &TilemapAnchor,
        &TiledMapLayerZOffset,
        &TilemapRenderSettings,
        &mut TiledWorldStorage,
    )>,
) {
    for (
        world_entity,
        world_handle,
        world_transform,
        world_chunking,
        anchor,
        layer_offset,
        render_settings,
        mut storage,
    ) in world_query.iter_mut()
    {
        // Make sure we have a valid reference on a fully loaded world asset
        let Some(tiled_world) = asset_server
            .get_recursive_dependency_load_state(&world_handle.0)
            .and_then(|state| {
                if state.is_loaded() {
                    return worlds.get(&world_handle.0);
                }
                None
            })
        else {
            continue;
        };

        let mut to_remove = Vec::new();
        let mut to_spawn = Vec::new();

        if let Some(chunking) = world_chunking.0 {
            let mut visible_maps = Vec::new();
            let cameras: Vec<Aabb2d> = camera_query
                .iter()
                .map(|transform| {
                    Aabb2d::new(
                        Vec2::new(transform.translation.x, transform.translation.y),
                        chunking,
                    )
                })
                .collect();
            // Check which map is visible by testing them against each camera (if there are multiple)
            // If map aabb overlaps with the camera_view, it is visible
            tiled_world.for_each_map(world_transform, anchor, |idx, aabb| {
                for c in cameras.iter() {
                    if aabb.intersects(c) {
                        visible_maps.push(idx);
                    }
                }
            });

            // All the maps that are visible but not already spawned should be spawned
            for idx in visible_maps.iter() {
                if !storage.maps.contains_key(idx) {
                    to_spawn.push(*idx);
                }
            }

            // All the maps that are spawned but not visible should be removed
            for (idx, _) in storage.maps.iter() {
                if !visible_maps.iter().any(|i| i == idx) {
                    to_remove.push(*idx);
                }
            }
        } else if storage.maps.is_empty() {
            // No chunking and we don't have spawned any map yet: just spawn all maps
            for idx in 0..tiled_world.maps.len() {
                to_spawn.push(idx as u32);
            }
        }

        // Despawn maps
        for idx in to_remove {
            if let Some(map_entity) = storage.maps.remove(&idx) {
                debug!("Despawn map (index = {}, entity = {:?})", idx, map_entity);
                commands.entity(map_entity).despawn();
            }
        }

        // Spawn maps
        let offset = tiled_world.offset(anchor);
        for idx in to_spawn {
            let Some((rect, handle)) = tiled_world.maps.get(idx as usize) else {
                continue;
            };
            let map_entity = commands
                .spawn((
                    ChildOf(world_entity),
                    TiledMap(handle.clone_weak()),
                    Transform::from_translation(
                        offset.extend(0.0) + Vec3::new(rect.min.x, rect.max.y, 0.0),
                    ),
                    // Force map anchor to TopLeft: everything is handled at
                    // world level. This makes it so each map's
                    // `Transform.translation` will have the same values for `x`
                    // and `y` that Tiled uses in its FILE.world.
                    TilemapAnchor::TopLeft,
                    *layer_offset,
                    *render_settings,
                ))
                .id();
            debug!(
                "Spawn map (index = {}, handle = {:?},  entity = {:?})",
                idx, handle, map_entity
            );
            storage.maps.insert(idx, map_entity);
        }
    }
}
