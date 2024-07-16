pub mod loader;

#[cfg(feature = "rapier")]
pub mod physics_rapier;

pub mod prelude {
    pub use crate::loader::*;
}
