pub mod asset;
pub mod components;
pub mod events;

/// `bevy_ecs_tiled` world related public exports
pub mod prelude {
    pub use super::asset::*;
    pub use super::components::*;
    pub use super::events::*;
}

use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapRenderSettings;

pub(crate) fn world_chunking(
    camera: Query<&Transform, (With<Camera>, Changed<Transform>)>,
    worlds: Res<Assets<TiledWorld>>,
    mut commands: Commands,
    mut world_query: Query<(
        Entity,
        &TiledWorldHandle,
        &GlobalTransform,
        &TiledWorldSettings,
        &TiledMapSettings,
        &TilemapRenderSettings,
        &mut TiledWorldStorage,
    )>,
) {
    for (
        world_entity,
        world_handle,
        world_transform,
        world_settings,
        map_settings,
        render_settings,
        mut storage,
    ) in world_query.iter_mut()
    {
        let Some(tiled_world) = worlds.get(&world_handle.0) else {
            return;
        };

        let world_position = Vec2::new(
            world_transform.translation().x,
            world_transform.translation().y,
        );

        let mut to_remove = Vec::new();
        let mut to_spawn = Vec::new();

        if let Some(chunking) = world_settings.chunking {
            // Test maps against each camera if there are multiple
            for camera_transform in camera.iter() {
                let chunk_view = Rect::new(
                    camera_transform.translation.x - chunking.0 as f32,
                    camera_transform.translation.y - chunking.1 as f32,
                    camera_transform.translation.x + chunking.0 as f32,
                    camera_transform.translation.y + chunking.1 as f32,
                );

                // Iterate through the chunked maps and remove any that are not in the chunk rect
                for (idx, (_, rect)) in storage.spawned_maps.iter().enumerate() {
                    // If map_rect does not overlap at all with the chunk_view, remove it
                    if rect.min.x + world_position.x > chunk_view.max.x
                        || rect.min.y + world_position.y > chunk_view.max.y
                        || rect.max.x + world_position.x < chunk_view.min.x
                        || rect.max.y + world_position.y < chunk_view.min.y
                    {
                        to_remove.push(idx);
                    }
                }

                // Check if any maps need to be spawned
                for map in tiled_world.maps.iter() {
                    if storage.spawned_maps.iter().any(|(_, rect)| rect == &map.0) {
                        continue;
                    }

                    // If map_rect does not overlap at all with the chunk_view skip it
                    if map.0.min.x + world_position.x > chunk_view.max.x
                        || map.0.min.y + world_position.y > chunk_view.max.y
                        || map.0.max.x + world_position.x < chunk_view.min.x
                        || map.0.max.y + world_position.y < chunk_view.min.y
                    {
                        continue;
                    }

                    to_spawn.push(map);
                }
            }
        } else if storage.spawned_maps.is_empty() {
            // No chunking and we don't have spawned any map yet: just spawn all maps
            for map in tiled_world.maps.iter() {
                to_spawn.push(map);
            }
        }

        // Despawn maps
        for idx in to_remove.iter().rev() {
            log::info!("Despawning map at index {}", idx);
            let (map_entity, _) = storage.spawned_maps.swap_remove(*idx);
            commands.entity(map_entity).despawn_recursive();
        }

        // Spawn maps
        for map in to_spawn.iter() {
            let map_entity = commands
                .spawn((
                    TiledMapHandle(map.1.clone()),
                    Transform::from_translation(Vec3::new(map.0.min.x, map.0.min.y, 0.0)),
                    TiledMapSettings {
                        layer_positioning: LayerPositioning::TiledOffset,
                        ..*map_settings
                    },
                    *render_settings,
                ))
                .set_parent(world_entity)
                .id();

            storage.spawned_maps.push((map_entity, map.0));
        }
    }
}

/// System to spawn a world once it has been fully loaded.
#[allow(clippy::type_complexity)]
pub(crate) fn process_loaded_worlds(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    worlds: ResMut<Assets<TiledWorld>>,
    mut world_query: Query<
        (
            Entity,
            &TiledWorldHandle,
            &TiledMapSettings,
            &mut Transform,
            &mut TiledWorldStorage,
        ),
        Or<(Changed<TiledWorldHandle>, With<RespawnTiledWorld>)>,
    >,
) {
    for (world_entity, world_handle, map_settings, mut world_transform, mut world_storage) in
        world_query.iter_mut()
    {
        if let Some(load_state) = asset_server.get_recursive_dependency_load_state(&world_handle.0)
        {
            if !load_state.is_loaded() {
                // If not fully loaded yet, insert the 'Respawn' marker so we will try to load it at next frame
                commands.entity(world_entity).insert(RespawnTiledWorld);
                debug!(
                    "World '{}' is not fully loaded yet...",
                    world_handle.0.path().unwrap()
                );
                continue;
            }

            let tiled_world = worlds.get(&world_handle.0).unwrap();
            info!(
                "World '{}' has finished loading, spawn it",
                world_handle.0.path().unwrap()
            );

            // Clean world
            remove_maps(&mut commands, &mut world_storage);

            // Adjust world transform if needed
            if let LayerPositioning::Centered = map_settings.layer_positioning {
                world_transform.translation += Vec3::new(
                    -tiled_world.world_rect.max.x / 2.0,
                    -tiled_world.world_rect.max.y / 2.0,
                    0.0,
                );
            }

            // Remove the 'Respawn' marker and insert additional components
            commands
                .entity(world_entity)
                .insert((
                    Name::new(format!(
                        "TiledWorld: {}",
                        tiled_world.world.source.display()
                    )),
                    TiledWorldMarker,
                ))
                .remove::<RespawnTiledWorld>();

            commands.trigger(TiledWorldCreated {
                world: world_entity,
                world_handle: world_handle.0.clone(),
            });
        }
    }
}

/// System to update worlds as they are changed or removed.
pub(crate) fn handle_world_events(
    mut commands: Commands,
    mut world_events: EventReader<AssetEvent<TiledWorld>>,
    mut world_query: Query<(Entity, &TiledWorldHandle)>,
) {
    for event in world_events.read() {
        match event {
            AssetEvent::Modified { id } => {
                info!("World changed: {id}");
                for (world_entity, world_handle) in world_query.iter() {
                    if world_handle.0.id() == *id {
                        commands.entity(world_entity).insert(RespawnTiledWorld);
                    }
                }
            }
            AssetEvent::Removed { id } => {
                info!("World removed: {id}");
                for (world_entity, world_handle) in world_query.iter_mut() {
                    if world_handle.0.id() == *id {
                        commands.entity(world_entity).despawn_recursive();
                    }
                }
            }
            _ => continue,
        }
    }
}

fn remove_maps(commands: &mut Commands, world_storage: &mut TiledWorldStorage) {
    for (map_entity, _) in world_storage.spawned_maps.iter() {
        commands.entity(*map_entity).despawn_recursive();
    }
    world_storage.spawned_maps.clear();
}
