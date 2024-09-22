//! This module handles all things related to Tiled custom properties.
//!
//! It is only available when the `user_properties` feature is enabled.
//!
//! See the [associated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/user_properties.rs) or the [dedicated book section](https://adrien-bon.github.io/bevy_ecs_tiled/guides/properties.html) for more information.

pub mod import;
pub mod load;
pub mod export;
pub mod command;

/// Re-exports, notably derive macros
pub mod prelude {
    pub use crate::properties::*;
}
