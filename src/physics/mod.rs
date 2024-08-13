#[cfg(feature = "rapier")]
pub mod rapier;

#[cfg(feature = "avian")]
pub mod avian;

pub mod prelude {
    #[cfg(feature = "avian")]
    pub use crate::physics::avian::*;
    #[cfg(feature = "rapier")]
    pub use crate::physics::rapier::*;
}
