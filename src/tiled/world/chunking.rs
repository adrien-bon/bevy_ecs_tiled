//! Chunk management for Tiled worlds.
//!
//! This module implements logic spawning and despawning Tiled maps based on camera position
//! and chunking configuration. It allows for efficient rendering and memory management by only
//! keeping visible maps in memory, while removing those that are not currently in view.

use crate::prelude::*;
use bevy::{
    prelude::*,
    shape::{Aabb2d, IntersectsVolume},
};

/// [`Component`] holding Tiled world chunking configuration.
///
/// If this value is None, we won't perform chunking: all maps from this world will just be loaded
/// If this value is set, defines the area (in pixel) around each [`Camera`] where we should spawn a
/// map if it overlaps with its associated [`Rect`].
///
/// Must be added to the [`Entity`] holding the world.
#[derive(Component, Default, Reflect, Copy, Clone, Debug, Deref)]
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
    // Do not filter on `Changed<Transform>`: we need all cameras every frame, otherwise
    // a frame where no camera moved would be seen as "no map visible" and despawn every map.
    camera_query: Query<&Transform, With<Camera>>,
    worlds: Res<Assets<TiledWorldAsset>>,
    maps: Res<Assets<TiledMapAsset>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut world_query: Query<(
        Entity,
        &TiledWorld,
        &GlobalTransform,
        &TiledWorldChunking,
        &TilemapAnchor,
        &TiledMapLayerZOffset,
        &TiledMapImageRepeatMargin,
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
        image_repeat_margin,
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
                to_spawn.push(WorldMapId(idx as u32));
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
            let Some((rect, handle)) = tiled_world.maps.get(idx.0 as usize) else {
                continue;
            };
            // Force map anchor to TopLeft: everything is handled at world level.
            // Depending on the map orientation and its tiles size, `TilemapAnchor::TopLeft`
            // does not always put the top-left corner of the map, as Tiled sees it, at the
            // map origin (for instance, isometric maps with tiles larger than the grid):
            // compensate so the map exactly covers the rect defined in the `.world` file.
            let anchor = TilemapAnchor::TopLeft;
            let map_origin = maps
                .get(handle)
                .map(|map_asset| map_asset.screen_origin(&anchor))
                .unwrap_or_default();
            let map_entity = commands
                .spawn((
                    ChildOf(world_entity),
                    TiledMap(handle.clone()),
                    Transform::from_translation(
                        (offset + Vec2::new(rect.min.x, rect.max.y) - map_origin).extend(0.0),
                    ),
                    anchor,
                    *layer_offset,
                    *image_repeat_margin,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiled::world::tests::load_world;

    fn chunking_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_plugins((
            crate::tiled::cache::plugin,
            crate::tiled::map::asset::plugin,
            crate::tiled::map::loader::plugin,
            crate::tiled::world::asset::plugin,
            crate::tiled::world::loader::plugin,
        ));
        app.add_systems(PostUpdate, handle_world_chunking);
        app
    }

    #[test]
    fn static_camera_keeps_visible_maps_spawned() {
        let mut app = chunking_app();
        let handle = load_world(&mut app, "tests/chunking/two_maps.world");

        // Camera at origin only overlaps the first map (second one is 1000px away)
        app.world_mut()
            .spawn((Camera::default(), Transform::default()));
        let world_entity = app
            .world_mut()
            .spawn((TiledWorld(handle), TiledWorldChunking::new(100., 100.)))
            .id();

        // The camera never moves: maps must stay spawned over several frames
        for frame in 0..5 {
            app.update();
            let storage = app
                .world()
                .get::<TiledWorldStorage>(world_entity)
                .expect("world entity should have a storage");
            assert_eq!(
                storage.maps.keys().copied().collect::<Vec<_>>(),
                vec![WorldMapId(0)],
                "unexpected spawned maps at frame {frame}"
            );
        }
    }
}
