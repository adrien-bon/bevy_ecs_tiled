//! Tiled maps and worlds as ECS entities.
//!
//! Entities spawned from a Tiled map hold its data (maps, layers, tiles, objects, images): they are
//! the source of truth for the map content. You can freely modify them, what is derived from
//! them (rendering, physics colliders) follows.

pub mod event;
pub mod filter;
pub mod image;
pub mod layer;
pub mod map;
pub mod object;
pub mod tile;
#[cfg(feature = "world")]
pub mod world;

use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        map::plugin,
        #[cfg(feature = "world")]
        world::plugin,
        event::plugin,
        image::plugin,
        layer::plugin,
        object::plugin,
        tile::plugin,
        filter::plugin,
    ));
}
