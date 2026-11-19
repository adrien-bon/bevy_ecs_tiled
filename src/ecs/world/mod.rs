//! Tiled worlds: spawn a [`TiledWorld`] to load all the maps of a `.world` file.
//!
//! Use [`TiledWorldStreaming`] to only spawn the maps around the camera, and [`TiledWorldMaps`] to retrieve
//! the spawned maps of a world.
//!
//! Only available with the `world` feature. See the [FAQ](https://adrien-bon.github.io/bevy_ecs_tiled/FAQ.html).

pub mod query;
pub mod streaming;

#[cfg(test)]
mod tests;

use crate::{ecs::event::TiledMessageWriters, prelude::*};
use bevy::{asset::RecursiveDependencyLoadState, prelude::*};

/// Main component for loading and managing a Tiled world in the ECS world.
///
/// Attach this component to an entity to load a Tiled world from a `.world` file. The inner value is a [`Handle<TiledWorldAsset>`],
/// which references the loaded [`TiledWorldAsset`]. This entity acts as the root for all maps, layers, and objects spawned from the world.
///
/// Required components (automatically added with default value if missing):
/// - [`TiledWorldStreaming`]: Controls which maps of the world are spawned, around cameras.
/// - the settings components of a [`TiledMap`] (see its documentation): they apply to the whole world
///   and are passed on to its maps, except [`TilemapAnchor`] which applies to the world itself.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn spawn_world(mut commands: Commands, asset_server: Res<AssetServer>) {
///     commands.spawn(TiledWorld(asset_server.load("demo.world")));
/// }
/// ```
#[derive(Component, Reflect, Clone, Debug, Deref, FromTemplate)]
#[reflect(Component, Debug)]
#[require(TiledWorldStreaming, crate::ecs::map::TiledMapSettings)]
pub struct TiledWorld(pub Handle<TiledWorldAsset>);

/// Settings components of a [`TiledWorld`] which are passed on to each of its maps.
pub(crate) type TiledWorldInheritedSettings = (
    &'static TiledMapLayerZOffset,
    &'static TiledMapImageRepeatMargin,
    &'static TilemapRenderSettings,
);

/// Filter matching worlds whose [`TiledWorldInheritedSettings`] or [`TilemapAnchor`] changed.
type TiledWorldSettingsChanged = Or<(
    Changed<TilemapAnchor>,
    Changed<TiledMapLayerZOffset>,
    Changed<TiledMapImageRepeatMargin>,
    Changed<TilemapRenderSettings>,
)>;

/// Components to insert on a map of a world so it inherits the world settings.
pub(crate) fn inherited_settings(
    (layer_offset, image_repeat_margin, render_settings): (
        &TiledMapLayerZOffset,
        &TiledMapImageRepeatMargin,
        &TilemapRenderSettings,
    ),
) -> impl Bundle {
    (*layer_offset, *image_repeat_margin, *render_settings)
}

/// Marker component to trigger a Tiled world respawn.
///
/// Add this component to the entity holding the [`TiledWorld`] to force the world and all its maps to be reloaded.
/// This is useful for hot-reloading, resetting, or programmatically refreshing the world state.
///
/// When present, the plugin will despawn all child entities and re-instantiate the world from its asset, preserving the top-level entity and its components.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn respawn_world(mut commands: Commands, world_query: Query<Entity, With<TiledWorld>>) {
///     if let Ok(entity) = world_query.single() {
///         commands.entity(entity).insert(RespawnTiledWorld);
///     }
/// }
/// ```
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

    app.add_plugins((query::plugin, streaming::plugin));
}

/// System to spawn a world once it has been fully loaded.
fn process_loaded_worlds(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    worlds: Res<Assets<TiledWorldAsset>>,
    world_query: Query<
        (Entity, &TiledWorld),
        Or<(
            Changed<TiledWorld>,
            // If a world settings change, force a respawn so they can be taken into account
            TiledWorldSettingsChanged,
            With<RespawnTiledWorld>,
            // Not needed to react to changes on TiledWorldStreaming:
            // it's read each frame by world_streaming() system
        )>,
    >,
    world_maps: TiledWorldMaps<Entity>,
    mut message_writers: TiledMessageWriters,
) {
    for (world_entity, world_handle) in world_query.iter() {
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
                error!(
                    "Cannot get a valid TiledWorld out of Handle<TiledWorld>: has the last strong reference to the asset been dropped ? (handle = {:?} / entity = {:?})",
                    world_handle.0, world_entity
                );
                commands.entity(world_entity).despawn();
                continue;
            };

            debug!(
                "World has finished loading, spawn world maps (handle = {:?})",
                world_handle.0
            );

            // Clean previous maps before trying to spawn the new ones
            for map in world_maps.iter(world_entity) {
                commands.entity(map).despawn();
            }

            // Remove the 'Respawn' marker and insert additional components
            // Actual map spawn is handled by world_streaming() system
            commands
                .entity(world_entity)
                .insert(Name::new(format!(
                    "TiledWorld: {}",
                    tiled_world.world.source.display()
                )))
                .remove::<RespawnTiledWorld>();

            TiledEvent::new(world_entity, WorldCreated)
                .with_world(world_entity, world_handle.0.id())
                .send(&mut commands, &mut message_writers.world_created);
        }
    }
}

/// System to update worlds as they are changed or removed.
fn handle_world_events(
    mut commands: Commands,
    mut world_events: MessageReader<AssetEvent<TiledWorldAsset>>,
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
