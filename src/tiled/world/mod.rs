//! Tiled world management and logic.
//!
//! This module contains the core logic for handling Tiled worlds, including loading, chunking, and managing
//! multiple maps within a world. It organizes submodules and systems related to world storage, chunk visibility,
//! and world-level events, providing the main entry point for Tiled world support

pub mod asset;
pub mod chunking;
pub mod loader;
pub mod storage;

use crate::{prelude::*, tiled::event::TiledEventWriters};
use bevy::{asset::RecursiveDependencyLoadState, prelude::*};

/// Wrapper around the [`Handle`] to the `.world` file representing the [`TiledWorld`].
///
/// This is the main [`Component`] that must be spawned to load a Tiled world.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Debug)]
#[require(
    TiledWorldStorage,
    TiledWorldChunking,
    TiledMapLayerZOffset,
    TiledMapImageRepeatMargin,
    TilemapAnchor,
    TilemapRenderSettings,
    Visibility,
    Transform
)]
pub struct TiledWorld(pub Handle<TiledWorldAsset>);

/// Marker [`Component`] to trigger a world respawn.
///
/// Must be added to the [`Entity`] holding the map.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct RespawnTiledWorld;

pub(crate) fn plugin(app: &mut bevy::prelude::App) {
    app.register_type::<TiledWorld>();
    app.register_type::<RespawnTiledWorld>();
    app.add_systems(
        PreUpdate,
        process_loaded_worlds.in_set(TiledPreUpdateSystems::ProcessLoadedWorlds),
    );
    app.add_systems(
        PostUpdate,
        handle_world_events.in_set(TiledPostUpdateSystems::HandleWorldAssetEvents),
    );

    app.add_plugins((
        asset::plugin,
        loader::plugin,
        storage::plugin,
        chunking::plugin,
    ));
}

/// System to spawn a world once it has been fully loaded.
fn process_loaded_worlds(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    worlds: Res<Assets<TiledWorldAsset>>,
    mut world_query: Query<
        (Entity, &TiledWorld, &mut TiledWorldStorage),
        Or<(
            Changed<TiledWorld>,
            // If a world settings change, force a respawn so they can be taken into account
            Changed<TilemapAnchor>,
            Changed<TiledMapLayerZOffset>,
            Changed<TiledMapImageRepeatMargin>,
            Changed<TilemapRenderSettings>,
            With<RespawnTiledWorld>,
            // Not needed to react to changes on TiledWorldChunking:
            // it's read each frame by world_chunking() system
        )>,
    >,
    mut event_writers: TiledEventWriters,
) {
    for (world_entity, world_handle, mut world_storage) in world_query.iter_mut() {
        if let Some(load_state) = asset_server.get_recursive_dependency_load_state(&world_handle.0)
        {
            if !load_state.is_loaded() {
                if let RecursiveDependencyLoadState::Failed(_) = load_state {
                    error!(
                        "World failed to load, despawn it (handle = {:?} / entity = {:?})",
                        world_handle.0, world_entity
                    );
                    commands.entity(world_entity).despawn();
                } else {
                    // If not fully loaded yet, insert the 'Respawn' marker so we will try to load it at next frame
                    debug!(
                        "World is not fully loaded yet, will try again next frame (handle = {:?} / entity = {:?})",
                        world_handle.0, world_entity
                    );
                    commands.entity(world_entity).insert(RespawnTiledWorld);
                }
                continue;
            }

            // World should be loaded at this point
            let Some(tiled_world) = worlds.get(&world_handle.0) else {
                error!("Cannot get a valid TiledWorld out of Handle<TiledWorld>: has the last strong reference to the asset been dropped ? (handle = {:?} / entity = {:?})", world_handle.0, world_entity);
                commands.entity(world_entity).despawn();
                continue;
            };

            debug!(
                "World has finished loading, spawn world maps (handle = {:?})",
                world_handle.0
            );

            // Clean previous maps before trying to spawn the new ones
            world_storage.clear(&mut commands);

            // Remove the 'Respawn' marker and insert additional components
            // Actual map spawn is handled by world_chunking() system
            commands
                .entity(world_entity)
                .insert(Name::new(format!(
                    "TiledWorld: {}",
                    tiled_world.world.source.display()
                )))
                .remove::<RespawnTiledWorld>();

            TiledEvent::new(world_entity, WorldCreated)
                .with_world(world_entity, world_handle.0.id())
                .send(&mut commands, &mut event_writers.world_created);
        }
    }
}

/// System to update worlds as they are changed or removed.
fn handle_world_events(
    mut commands: Commands,
    mut world_events: EventReader<AssetEvent<TiledWorldAsset>>,
    world_query: Query<(Entity, &TiledWorld)>,
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
                for (world_entity, world_handle) in world_query.iter() {
                    if world_handle.0.id() == *id {
                        commands.entity(world_entity).despawn();
                    }
                }
            }
            _ => continue,
        }
    }
}
