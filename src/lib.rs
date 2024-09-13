#![doc = include_str!("../book/src/intro.md")]
//!
//! ## Getting started
//!
#![doc = include_str!("../book/src/getting-started.md")]
//!
//! ## API reference
//!
//! As the name implies, this API reference documentation purpose is to describe the API provided by `bevy_ecs_tiled`.
//! A good entry-point would be the [components::TiledMapBundle] component which is the component used to actually spawn a map.
//!
//! For a more use-cases oriented documentation please have a look to the [`bevy_ecs_tiled` book](https://adrien-bon.github.io/bevy_ecs_tiled/).
//!

pub mod components;
pub mod debug;
pub mod loader;
pub mod names;
pub mod physics;
pub mod utils;

#[cfg(feature = "user_properties")]
pub mod properties;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::debug::*;
    pub use crate::loader::*;
    pub use crate::names::*;
    pub use crate::physics::*;
    #[cfg(feature = "user_properties")]
    pub use crate::properties::prelude::*;
    pub use crate::utils::*;
    pub use tiled;
}
