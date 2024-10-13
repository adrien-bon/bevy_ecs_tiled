//! This example will delay map spawn from asset loading to demonstrate both are decoupled.

use std::time::*;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // Examples helper plugin (does not matter for this example)
        .add_plugins(helper::HelperPlugin)
        // bevy_ecs_tilemap and bevy_ecs_tiled main plugins
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin::default())
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, spawn_map)
        .run();
}

#[derive(Resource)]
struct MapSpawner {
    map_handle: Handle<TiledMap>,
    timer: Timer,
}

const DELAY_VALUE_S: u64 = 5;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    info!("Loading asset, will spawn a map in {}s ...", DELAY_VALUE_S);
    commands.insert_resource(MapSpawner {
        map_handle: asset_server.load("finite.tmx"),
        timer: Timer::new(Duration::from_secs(DELAY_VALUE_S), TimerMode::Once),
    });
}

fn spawn_map(mut commands: Commands, mut spawner: ResMut<MapSpawner>, time: Res<Time>) {
    spawner.timer.tick(time.delta());
    if spawner.timer.just_finished() {
        info!("Timer finished, spawn the map !");
        commands.spawn(
            TiledMapHandle(spawner.map_handle.clone()),
        );
    }
}
