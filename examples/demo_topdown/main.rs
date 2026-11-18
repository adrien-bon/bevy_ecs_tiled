//! A small top-down game, to showcase how to use `bevy_ecs_tiled` in an actual game.
//!
//! The player is moved with the mouse on a grid, without any physics engine: everything
//! the game needs to know about the world comes from user properties set in Tiled.
//! - `level`: load the maps and move the player from one map to another through doors
//! - `navigation`: build a navigation grid from tiles properties and find paths on it
//! - `picking`: a picking backend for tiles, so they receive regular pointer events
//! - `player`: spawn the player and move it along its path when a tile is clicked
//! - `interaction`: destructible crates, buttons and gates, which modify the map tiles
//! - `highlight`: a custom tilemap material to highlight some tiles
//! - `camera`: not specific to Tiled
//!
//! See the `README.md` file next to this one for more information.

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_ecs_tiled::prelude::*;

mod camera;
mod highlight;
mod interaction;
mod level;
mod navigation;
mod picking;
mod player;

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
                    title: "Top-down Demo".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            }),
    );

    // Order our `Update` systems: handle what happened on the tile
    // the player stands on before moving it to the next one.
    app.configure_sets(
        Update,
        (
            UpdateSystems::Navigation,
            UpdateSystems::Interact,
            UpdateSystems::Move,
            UpdateSystems::Display,
        )
            .chain(),
    );

    #[cfg(feature = "debug")]
    {
        use bevy::input::common_conditions::input_toggle_active;
        use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

        // Toggle the world inspector with F1, to have a look at components added from Tiled
        app.add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)),
        ));
    }

    let mut config = TiledPluginConfig::default();

    // If WASM is enabled, do not export user properties
    // Note: properties will still be loaded from the map
    #[cfg(feature = "wasm")]
    {
        config.types_export_file = None;
    }

    // Only export properties when WASM is disabled
    #[cfg(not(feature = "wasm"))]
    {
        // Custom directory for exporting user properties
        let mut path = std::env::current_dir().unwrap();
        path.push("exports");
        path.push("demo_topdown_types.json");
        config.types_export_file = Some(path);

        // Custom filter to chose which properties should be exported
        config.types_filter = TiledFilter::from(
            bevy_ecs_tiled::regex::RegexSet::new([r"^demo_topdown::.*"]).unwrap(),
        );
    }

    // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
    // be automatically added as well if it's not already done.
    app.add_plugins(TiledPlugin(config));

    // Add our own plugins: our custom tilemap material requires
    // the tilemap rendering to be already there.
    app.add_plugins((
        camera::plugin,
        highlight::plugin,
        interaction::plugin,
        level::plugin,
        navigation::plugin,
        picking::plugin,
        player::plugin,
    ));

    // Set camera default clear color
    app.insert_resource(ClearColor(Color::srgb_u8(46, 52, 64)));

    // Run the app !
    app.run();
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum UpdateSystems {
    /// Update the navigation grid from the map tiles.
    Navigation,
    /// React to the tile the player stands on.
    Interact,
    /// Move the player along its path.
    Move,
    /// Update what is displayed.
    Display,
}
