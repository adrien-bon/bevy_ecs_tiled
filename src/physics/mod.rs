#[cfg(feature = "rapier")]
pub mod rapier;

pub mod prelude {
    #[cfg(feature = "rapier")]
    pub use crate::physics::rapier::*;
}
