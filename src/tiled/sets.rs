//! System sets for the bevy_ecs_tiled plugin.
//!
//! This module defines enums grouping related systems for the PreUpdate, Update, and PostUpdate
//! schedules. These sets help organize and order the execution of systems related to Tiled map and
//! world processing, including asset loading, physics initialization, animation, debugging, and
//! chunk management.

use bevy::prelude::*;

/// System sets for the PreUpdate schedule.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum TiledPreUpdateSystems {
    /// Marker for the first system in the pre-update phase.
    First,
    /// Processes loaded worlds before maps.
    ProcessLoadedWorlds,
    /// Processes loaded maps after worlds.
    ProcessLoadedMaps,
    /// Initializes physics settings for Tiled maps and worlds.
    InitializePhysicsSettings,
    /// Marker for the last system in the pre-update phase.
    Last,
}

/// System sets for the Update schedule.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum TiledUpdateSystems {
    /// Marker for the first system in the update phase.
    First,
    /// Animates Tiled sprites.
    AnimateSprite,
    /// Updates parallax layers.
    UpdateParallaxLayers,
    /// Update Tiled images position and size.
    UpdateTiledImagePositionAndSize,
    /// Runs debug systems related to Tiled maps and worlds.
    Debug,
    /// Marker for the last system in the update phase.
    Last,
}

/// System sets for the PostUpdate schedule.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum TiledPostUpdateSystems {
    /// Marker for the first system in the post-update phase.
    First,
    /// Handle any physics settings update by respawning either the world or the map.
    HandlePhysicsSettingsUpdate,
    /// Handles asset events for Tiled worlds.
    HandleWorldAssetEvents,
    /// Handles chunking of Tiled worlds by spawning or despawning maps based on their visibility.
    HandleWorldChunking,
    /// Handles asset events for Tiled maps.
    HandleMapAssetEvents,
    /// Keeps [`TiledMapStorage`](crate::prelude::TiledMapStorage) in sync with added or modified tiles.
    SyncMapStorage,
    /// Creates or updates rendering components (tilemaps, sprites, texts) from Tiled data components.
    SyncRender,
    /// Creates or updates physics colliders from Tiled data components.
    SyncPhysics,
    /// Marker for the last system in the post-update phase.
    Last,
}

pub(crate) fn plugin(app: &mut App) {
    app.configure_sets(
        PreUpdate,
        (
            TiledPreUpdateSystems::First,
            TiledPreUpdateSystems::ProcessLoadedWorlds,
            TiledPreUpdateSystems::ProcessLoadedMaps,
            TiledPreUpdateSystems::InitializePhysicsSettings,
            TiledPreUpdateSystems::Last,
        )
            .chain(),
    );
    app.configure_sets(
        Update,
        (
            TiledUpdateSystems::First,
            TiledUpdateSystems::AnimateSprite,
            TiledUpdateSystems::UpdateParallaxLayers,
            TiledUpdateSystems::UpdateTiledImagePositionAndSize,
            TiledUpdateSystems::Debug,
            TiledUpdateSystems::Last,
        )
            .chain(),
    );
    app.configure_sets(
        PostUpdate,
        (
            TiledPostUpdateSystems::First,
            TiledPostUpdateSystems::HandlePhysicsSettingsUpdate,
            TiledPostUpdateSystems::HandleWorldAssetEvents,
            TiledPostUpdateSystems::HandleWorldChunking,
            TiledPostUpdateSystems::HandleMapAssetEvents,
            TiledPostUpdateSystems::SyncMapStorage,
            TiledPostUpdateSystems::SyncRender,
            TiledPostUpdateSystems::SyncPhysics,
            TiledPostUpdateSystems::Last,
        )
            .chain(),
    );
    // Rendering and physics components derived from Tiled data must be
    // up to date before computing transforms and visibility
    app.configure_sets(
        PostUpdate,
        (
            TiledPostUpdateSystems::SyncRender,
            TiledPostUpdateSystems::SyncPhysics,
        )
            .before(bevy::transform::TransformSystems::Propagate)
            .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
    );
}
