//! This example demonstrates how to load and unload maps.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(helper::HelperPlugin)
        .init_state::<MapState>()
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                handle_load.run_if(in_state(MapState::Unloaded)),
                (handle_unload, handle_reload).run_if(in_state(MapState::Loaded)),
            ),
        )
        .add_systems(Update, log_transitions)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<MapState>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TextBundle::from(
        "U = Unload map by removing asset\nI = Unload map by despawning entity\nL = Load finite map\nK = Replace loaded map component without unloading",
    ));

    let map_handle: Handle<TiledMap> = asset_server.load("finite.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
    next_state.set(MapState::Loaded);
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum MapState {
    Loaded,
    #[default]
    Unloaded,
}

fn handle_load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<MapState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        let map_handle: Handle<TiledMap> = asset_server.load("finite.tmx");
        commands.spawn(TiledMapBundle {
            tiled_map: map_handle,
            ..Default::default()
        });
        next_state.set(MapState::Loaded);
    }
}

fn handle_reload(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    maps_query: Query<(Entity, &Handle<TiledMap>)>,
    mut next_state: ResMut<NextState<MapState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyK) {
        let map_handle: Handle<TiledMap> = asset_server.load("infinite.tmx");
        let (entity, _) = maps_query.single();
        commands.entity(entity).insert(TiledMapBundle {
            tiled_map: map_handle,
            ..Default::default()
        });

        next_state.set(MapState::Loaded);
    }
}

fn handle_unload(
    mut commands: Commands,
    mut maps: ResMut<Assets<TiledMap>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    maps_query: Query<(Entity, &Handle<TiledMap>)>,
    mut next_state: ResMut<NextState<MapState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyU) {
        // This example shows that the map gets properly unloaded if the
        // `TiledMap` asset is removed.
        //
        // However, typically you would remove the map entity instead.
        let handles: Vec<_> = maps.iter().map(|(handle, _)| handle).collect();
        for handle in handles {
            // This will cause the map to unload.
            maps.remove(handle);
        }

        // Actually remove the entities, so that we can re-add later.
        // If we don't do this, the entity still exists and the map will not be
        // reloaded properly.
        for map in maps_query.iter() {
            commands.entity(map.0).despawn_recursive();
        }
        next_state.set(MapState::Unloaded);
    } else if keyboard_input.just_pressed(KeyCode::KeyI) {
        // Just remove the entities directly. This will also unload the map.
        for map in maps_query.iter() {
            commands.entity(map.0).despawn_recursive();
        }
        next_state.set(MapState::Unloaded);
    }
}

fn log_transitions(mut transitions: EventReader<StateTransitionEvent<MapState>>) {
    for transition in transitions.read() {
        info!(
            "transition: {:?} => {:?}",
            transition.exited, transition.entered
        );
    }
}
