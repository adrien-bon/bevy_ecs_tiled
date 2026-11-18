//! Tiled custom properties.
//!
//! Custom properties of Tiled maps, layers and objects are always available as raw [`tiled`] data,
//! in the [`TiledProperties`] component.
//!
//! With the `user_properties` feature, they are also deserialized as your own [`Reflect`] components and
//! resources, and the types registered in your application can be exported to be imported in Tiled.
//! See the [associated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/properties_basic.rs)
//! or the [dedicated book section](https://adrien-bon.github.io/bevy_ecs_tiled/guides/properties.html) for more information.

mod raw;
#[cfg(feature = "user_properties")]
pub(crate) mod user;

pub use raw::TiledProperties;
#[cfg(feature = "user_properties")]
pub use user::export_types;

use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((
        raw::plugin,
        #[cfg(feature = "user_properties")]
        user::plugin,
    ));
}
