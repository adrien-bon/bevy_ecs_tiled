#![doc = include_str!("../book/src/intro.md")]
//!
//! ## API reference
//!
//! As the name implies, this API reference documentation purpose is to describe the API provided by `bevy_ecs_tiled`.
//! A good entry-point would be the [TiledMapBundle](components::TiledMapBundle) component which is the `Component` used to spawn a map.
//!
//! For a more use-cases oriented documentation please have a look to the [`bevy_ecs_tiled` book](https://adrien-bon.github.io/bevy_ecs_tiled/).
//!
//! ## Getting started
//!
#![doc = include_str!("../book/src/getting-started.md")]

pub mod components;
pub mod debug;
pub mod events;
pub mod loader;
pub mod names;
pub mod physics;
pub mod utils;

#[cfg(feature = "user_properties")]
pub mod properties;

/// `bevy_ecs_tiled` public exports.
pub mod prelude {
    pub use crate::components::*;
    pub use crate::debug::*;
    pub use crate::events::*;
    pub use crate::loader::*;
    pub use crate::names::*;
    pub use crate::physics::*;
    #[cfg(feature = "user_properties")]
    pub use crate::properties::prelude::*;
    pub use crate::utils::*;
    pub use tiled;
}
