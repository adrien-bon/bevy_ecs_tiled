//! This module handles all things related to Tiled custom properties.
//! 
//! It is only available when the `user_properties` feature is enabled.
//! 
//! See the [associated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/user_properties.rs) or the [dedicated book section](https://adrien-bon.github.io/bevy_ecs_tiled/guides/properties.html) for more information.

pub mod app;
pub mod events;
pub mod traits;

/// Re-exports, notably derive macros
pub mod prelude {
    pub use crate::properties::app::*;
    pub use crate::properties::events::*;
    pub use crate::properties::traits::*;
    pub use crate::properties::*;
    #[doc(inline)]
    pub use bevy_ecs_tiled_macros::*;
}

/// Conversion methods between Tiled types and Rust types.
pub trait IntoUserType<T> {
    fn into_user_type(self) -> T;
}

impl IntoUserType<bevy::color::Color> for tiled::PropertyValue {
    fn into_user_type(self) -> bevy::color::Color {
        match self {
            Self::ColorValue(x) => bevy::color::Color::srgba(
                x.red as f32,
                x.green as f32,
                x.blue as f32,
                x.alpha as f32,
            ),
            _ => panic!("Expected ColorValue!"),
        }
    }
}

impl<T> IntoUserType<T> for tiled::PropertyValue
where
    T: crate::properties::traits::TiledClass,
{
    fn into_user_type(self) -> T {
        match self {
            Self::ClassValue {
                property_type: _,
                properties,
            } => T::create(&properties),
            _ => panic!("Expected ClassValue!"),
        }
    }
}

macro_rules! impl_into_user_type {
    ($ty:ty, $variant:ident) => {
        impl IntoUserType<$ty> for tiled::PropertyValue {
            fn into_user_type(self) -> $ty {
                match self {
                    Self::$variant(x) => x.clone(),
                    _ => panic!("Expected {}!", stringify!($variant)),
                }
            }
        }
    };
}

impl_into_user_type!(i32, IntValue);
impl_into_user_type!(f32, FloatValue);
impl_into_user_type!(bool, BoolValue);
impl_into_user_type!(String, StringValue);
impl_into_user_type!(u32, ObjectValue);
