pub mod loader;
pub mod names;
pub mod settings;

#[cfg(feature = "physics")]
pub mod physics;

pub mod prelude {
    pub use crate::loader::*;
    pub use crate::names::*;
    pub use crate::settings::*;
}
