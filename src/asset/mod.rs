//! Loading of Tiled maps and worlds as Bevy [`Asset`]s.
//!
//! This layer does not depend on the ECS world: it only reads Tiled files and exposes their raw
//! [`tiled`] data along with some conversions to Bevy and `bevy_ecs_tilemap` coordinates.

pub(crate) mod cache;
pub(crate) mod helpers;
mod ids;
pub mod map;
pub(crate) mod reader;
#[cfg(feature = "world")]
pub mod world;

#[cfg(feature = "world")]
pub use ids::WorldMapId;
pub use ids::{LayerId, ObjectId, TilesetId};

use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        // Must come first: asset loaders use the resource cache
        cache::plugin,
        ids::plugin,
        map::plugin,
        #[cfg(feature = "world")]
        world::plugin,
    ));
}
