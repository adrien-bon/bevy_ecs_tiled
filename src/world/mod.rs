pub mod asset;
pub mod components;
pub mod json;

/// `bevy_ecs_tiled` world related public exports
pub mod prelude {
    pub use super::asset::*;
    pub use super::components::*;
}

use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

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
            &TilemapRenderSettings,
            &TiledMapSettings,
        ),
        Or<(Changed<TiledWorldHandle>, With<RespawnTiledWorld>)>,
    >,
) {
    for (world_entity, world_handle, world_settings, mut world_storage, render_settings, tiled_settings) in
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

            // XXX: load the world, ie. update TiledWorldStorage component based upon informations from TiledWorldHandle / TiledWorld

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
