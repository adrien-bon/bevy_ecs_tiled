//! This example will delay map spawn from asset loading to demonstrate both are decoupled.

use std::time::*;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins: prevent blur effect by changing default sampling
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done
        .add_plugins(TiledMapPlugin::default())
        // Examples helper plugins, such as the logic to pan and zoom the camera
        // This should not be used directly in your game (but you can always have a look)
        .add_plugins(helper::HelperPlugin)
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
    commands.spawn(Camera2d);

    info!("Loading asset, will spawn a map in {}s ...", DELAY_VALUE_S);
    commands.insert_resource(MapSpawner {
        map_handle: asset_server.load("maps/orthogonal/finite.tmx"),
        timer: Timer::new(Duration::from_secs(DELAY_VALUE_S), TimerMode::Once),
    });
}

fn spawn_map(mut commands: Commands, mut spawner: ResMut<MapSpawner>, time: Res<Time>) {
    spawner.timer.tick(time.delta());
    if spawner.timer.just_finished() {
        info!("Timer finished, spawn the map !");
        commands.spawn(TiledMapHandle(spawner.map_handle.clone()));
    }
}
