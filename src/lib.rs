pub mod components;
pub mod loader;
pub mod names;

#[cfg(feature = "physics")]
pub mod physics;

#[cfg(feature = "user_properties")]
pub mod properties;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::loader::*;
    pub use crate::names::*;
    #[cfg(feature = "physics")]
    pub use crate::physics::*;
    #[cfg(feature = "user_properties")]
    pub use crate::properties::prelude::*;
    pub use tiled;
}
