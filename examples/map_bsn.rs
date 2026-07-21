//! This example shows the basic usage of the plugin with the Bevy bsn! macro.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins: prevent blur effect by changing default sampling
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done
        .add_plugins(TiledPlugin::default())
        // Examples helper plugins, such as the logic to pan and zoom the camera
        // This should not be used directly in your game (but you can always have a look)
        .add_plugins(helper::HelperPlugin)
        // Add our systems and run the app!
        .add_systems(Startup, level.spawn())
        .run();
}

fn level() -> impl SceneList {
    bsn_list![
        // Spawn a 2D camera (required by Bevy)
        Camera2d,
        // Tiled map
        tiled_map("maps/orthogonal/finite.tmx")
    ]
}

fn tiled_map(map_file: &'static str) -> impl Scene {
    bsn! {
            // Only the [`TiledMap`] component is actually required to spawn a map.
            TiledMap(map_file)
            // But you can add extra components to change the defaults settings and how
            // your map is actually displayed
    //        TilemapAnchor::Center() // unfortunately, components from bevy_ecs_tilemap are not yet BSN-ready
        }
}
