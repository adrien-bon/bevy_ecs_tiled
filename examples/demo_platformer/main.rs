//! A small platformer game, to showcase how to use `bevy_ecs_tiled` in an actual game.
//!
//! Everything is defined in the Tiled map and turned into game logic using user properties:
//! - `level`: load the map and configure its physics colliders
//! - `player`: spawn the player on its spawn point and handle keyboard inputs
//! - `enemy` and `patrol`: enemies walking along a route drawn in Tiled
//! - `trigger`: zones which kill or teleport the player and props
//! - `platform`: one-way platforms
//! - `prop`: dynamic physics objects, such as crates
//! - `controller`, `animation`, `camera`, `minimap`: not specific to Tiled
//!
//! See the `README.md` file next to this one for more information.

use avian2d::prelude::*;
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_ecs_tiled::prelude::*;

mod animation;
mod camera;
mod controller;
#[cfg(feature = "debug")]
mod debug;
mod enemy;
mod level;
mod minimap;
mod patrol;
mod platform;
mod player;
mod prop;
mod trigger;

fn main() {
    let mut app = App::new();

    // Add Bevy plugins.
    app.add_plugins(
        DefaultPlugins
            // Prevent blur effect by changing default sampling.
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics on web build on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Window {
                    title: "Platformer Demo".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            }),
    );

    // Order new `UpdateSystems` variants by adding them here:
    app.configure_sets(
        Update,
        (
            UpdateSystems::TickTimers,
            UpdateSystems::RecordInput,
            UpdateSystems::UpdateSprite,
        )
            .chain(),
    );

    app.add_plugins((
        animation::plugin,
        camera::plugin,
        #[cfg(feature = "debug")]
        debug::plugin,
        player::plugin,
        enemy::plugin,
        patrol::plugin,
        platform::plugin,
        prop::plugin,
        level::plugin,
        trigger::plugin,
        controller::plugin,
        minimap::plugin,
    ));

    // By default, the plugin does not export user properties.
    // Note: properties are loaded from the map whether they are exported or not
    #[allow(unused_mut)]
    let mut tiled_plugin = TiledPlugin::default();

    // Only export properties when WASM is disabled
    #[cfg(not(feature = "wasm"))]
    {
        // Custom directory for exporting user properties
        let mut path = std::env::current_dir().unwrap();
        path.push("exports");
        path.push("demo_platformer_types.json");
        tiled_plugin.types_export_file = Some(path);

        // Custom filter to chose which properties should be exported
        tiled_plugin.types_filter = TiledFilter::from(
            bevy_ecs_tiled::regex::RegexSet::new([r"^demo_platformer::.*"]).unwrap(),
        );
    }

    app.add_plugins((
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done.
        tiled_plugin,
        // Setup physics
        TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default(),
        PhysicsPlugins::default()
            .with_length_unit(100.0)
            // Custom collision hooks, used for one-way platforms
            .with_collision_hooks::<platform::PlatformerCollisionHooks>(),
    ));

    // Set camera default clear color
    app.insert_resource(ClearColor(Color::srgb_u8(196, 237, 240)));

    // Run the app !
    app.run();
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum UpdateSystems {
    TickTimers,
    RecordInput,
    UpdateSprite,
}
