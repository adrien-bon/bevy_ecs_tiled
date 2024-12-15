pub mod asset;
pub mod components;

/// `bevy_ecs_tiled` world related public exports
pub mod prelude {
    pub use super::asset::*;
    pub use super::components::*;
    pub use super::WorldChunkedMaps;
}

use crate::prelude::*;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct WorldChunkedMaps(pub Vec<(Entity, Rect)>);

pub(crate) fn world_chunking(
    camera: Query<&Transform, (With<Camera>, Changed<Transform>)>,    
    worlds: Res<Assets<TiledWorld>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut world_query: Query<(Entity, &TiledWorldHandle, &TiledWorldSettings)>,
    mut chunked_maps: ResMut<WorldChunkedMaps>,
) {
    let camera_transform = match camera.iter().next() {
        Some(camera) => camera,
        None => return,
    };

    for (world_entity, world_handle, world_settings) in world_query.iter_mut() {
        let tiled_world = worlds.get(&world_handle.0).unwrap();
        
        if !world_settings.chunking {
            return;
        }

        let chunk_view = Rect::new(
            camera_transform.translation.x - world_settings.chunking_width as f32,
            camera_transform.translation.y - world_settings.chunking_height as f32,
            camera_transform.translation.x + world_settings.chunking_width as f32,
            camera_transform.translation.y + world_settings.chunking_height as f32,
        );

        // Despawn maps not in the chunking view
        let mut to_remove = Vec::new();

        for (idx, (_, rect)) in chunked_maps.0.iter().enumerate() {
            // If map_rect does not overlap at all with the chunk_view, remove it
            if rect.min.x > chunk_view.max.x
                || rect.min.y > chunk_view.max.y
                || rect.max.x < chunk_view.min.x
                || rect.max.y < chunk_view.min.y
            {
                to_remove.push(idx);
            }
        }

        for idx in to_remove.iter().rev() {
            log::info!("Despawning map at index {}", idx);
            let (map_entity, _) = chunked_maps.0.swap_remove(*idx);
            commands.entity(map_entity).despawn_recursive();
        }

        // Get the path of the world asset, we need to prepend that to the map_path
        let world_path = world_handle.0.path().unwrap();

        // Spawn maps in the chunking view if they're not already spawned
        for map in tiled_world.maps.iter() {
            let map_path = map.0.clone();
            if chunked_maps.0.iter().any(|(_, rect)| rect == &map.1) {
                continue;
            }

            // If map_rect does not overlap at all with the chunk_view skip it
            if map.1.min.x > chunk_view.max.x
                || map.1.min.y > chunk_view.max.y
                || map.1.max.x < chunk_view.min.x
                || map.1.max.y < chunk_view.min.y
            {
                continue;
            }

            let map_path = world_path.path().parent().expect("Parent path not found").join(map_path.clone());

            let map_entity = commands
                .spawn_empty()
                .insert(TiledMapHandle(asset_server.load(map_path.clone())))
                .insert(RespawnTiledMap)
                .insert(Transform::from_translation(Vec3::new(map.1.min.x, map.1.min.y, 0.0)))
                .set_parent(world_entity)
                .id();

            chunked_maps.0.push((map_entity, map.1));
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
            &TiledWorldSettings,
            &mut TiledWorldStorage,
        ),
        Or<(Changed<TiledWorldHandle>, With<RespawnTiledWorld>)>,
    >,
) {
    for (world_entity, world_handle, world_settings, mut world_storage) in
        world_query.iter_mut()
    {
        if let Some(load_state) = asset_server.get_recursive_dependency_load_state(&world_handle.0) {
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

            if world_settings.chunking {
                return;                
            }

            // Get the relative path of the world asset 
            let world_parent_path = world_handle.0.path().unwrap().parent().expect("World parent path not found");
            let world_path = world_parent_path.path();

            // Load all the maps
            for map in tiled_world.maps.iter() {
                let map_path = world_path.join(map.0.clone());
                commands
                    .spawn_empty()
                    .insert(TiledMapHandle(asset_server.load(map_path)))
                    .insert(RespawnTiledMap)
                    .insert(Transform::from_translation(Vec3::new(map.1.min.x, map.1.min.y, 0.0)))
                    .set_parent(world_entity);
            }


            // Remove the respawn marker
            commands.entity(world_entity).remove::<RespawnTiledWorld>();
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
    for map in world_storage.0.iter() {
        if let Some(map_entity) = map.entity {
            commands.entity(map_entity).despawn_recursive();
        }
    }
    world_storage.0.clear();
}
