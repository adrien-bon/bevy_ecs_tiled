use avian2d::prelude::*;
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_ecs_tiled::prelude::*;

mod animation;
mod camera;
mod controller;
mod debug;
mod level;
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
            UpdateSystems::ApplyMovement,
            UpdateSystems::Update,
        )
            .chain(),
    );

    app.add_plugins((
        animation::plugin,
        camera::plugin,
        debug::plugin,
        player::plugin,
        level::plugin,
        controller::CharacterControllerPlugin,
    ));

    app
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done.
        .add_plugins((
            TiledPlugin(TiledPluginConfig {
                tiled_types_filter: TiledFilter::from(
                    RegexSet::new([r"^demo_platformer::.*"]).unwrap(),
                ),
                ..default()
            }),
            TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default(),
            PhysicsPlugins::default().with_length_unit(100.0),
        ));

    app.run();
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum UpdateSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    ApplyMovement,
    /// Do everything else (consider splitting this into further variants).
    Update,
}
