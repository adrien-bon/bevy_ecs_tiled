pub mod components;
pub mod debug;
pub mod loader;
pub mod names;
pub mod utils;

#[cfg(feature = "physics")]
pub mod physics;

#[cfg(feature = "user_properties")]
pub mod properties;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::debug::*;
    pub use crate::loader::*;
    pub use crate::names::*;
    #[cfg(feature = "physics")]
    pub use crate::physics::*;
    #[cfg(feature = "user_properties")]
    pub use crate::properties::prelude::*;
    pub use crate::utils::*;
    pub use tiled;
}
