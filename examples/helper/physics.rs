//! Physics backend used by the `physics_settings` and `physics_orientation` examples:
//! Avian with the `avian_debug` feature, Rapier with the `rapier_debug` feature.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

/// The physics backend in use
#[cfg(feature = "avian_debug")]
pub type Backend = TiledPhysicsAvianBackend;
/// The physics backend in use
#[cfg(all(feature = "rapier_debug", not(feature = "avian_debug")))]
pub type Backend = TiledPhysicsRapierBackend;

/// Add the `bevy_ecs_tiled` physics plugin, along with the physics engine and its debug rendering.
pub fn plugin(app: &mut App) {
    // This is where we select which physics backend to use, so colliders are automatically spawned
    app.add_plugins(TiledPhysicsPlugin::<Backend>::default());

    #[cfg(feature = "avian_debug")]
    {
        use avian2d::prelude::*;
        app.add_plugins((
            PhysicsPlugins::default().with_length_unit(100.0),
            PhysicsDebugPlugin,
        ));
    }

    #[cfg(all(feature = "rapier_debug", not(feature = "avian_debug")))]
    {
        use bevy_rapier2d::prelude::*;
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ));
    }
}
