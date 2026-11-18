//! Rendering of Tiled maps.
//!
//! Rendering entities and components are derived from the Tiled data held by [`ecs`](crate::ecs)
//! entities: they are created, updated and removed automatically, during
//! [`TiledPostUpdateSystems::SyncRender`](crate::sets::TiledPostUpdateSystems::SyncRender).
//!
//! This module is only available with the `render` feature.

pub mod animation;

use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(animation::plugin);
}
