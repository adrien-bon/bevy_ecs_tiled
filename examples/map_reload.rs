//! This example demonstrates how to load and unload maps.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // Examples helper plugin (does not matter for this example)
        .add_plugins(helper::HelperPlugin)
        // bevy_ecs_tiled main plugin
        .add_plugins(TiledMapPlugin::default())
        // Add our systems and run the app!
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
    commands.spawn(Camera2d);
    commands.spawn(Text::from(
        "U = Unload map by removing asset\nI = Unload map by despawning entity\nL = Load finite map\nK = Replace loaded map component without unloading\nR = Reload map using the RespawnTiledMap component",
    ));

    commands.spawn((
        TiledMapHandle(asset_server.load("maps/orthogonal/finite.tmx")),
        TiledMapSettings {
            layer_positioning: LayerPositioning::Centered,
            ..default()
        },
    ));
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
        info!("Load map");
        commands.spawn((
            TiledMapHandle(asset_server.load("maps/orthogonal/finite.tmx")),
            TiledMapSettings {
                layer_positioning: LayerPositioning::Centered,
                ..default()
            },
        ));
        next_state.set(MapState::Loaded);
    }
}

fn handle_reload(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    maps_query: Query<Entity, With<TiledMapMarker>>,
    mut next_state: ResMut<NextState<MapState>>,
) {
    // Reload the map by inserting a map asset on an existing map entity
    // Note that you can use the same map asset or a different one
    if keyboard_input.just_pressed(KeyCode::KeyK) {
        if let Ok(entity) = maps_query.get_single() {
            info!("Reload map");
            commands.entity(entity).insert(TiledMapHandle(
                asset_server.load("maps/orthogonal/infinite.tmx"),
            ));
            next_state.set(MapState::Loaded);
        } else {
            warn!("Cannot reload: no map loaded ?");
        }
    }

    // Reload the same map by pushing the RespawnTiledMap component on it
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        if let Ok(entity) = maps_query.get_single() {
            info!("Respawn map");
            commands.entity(entity).insert(RespawnTiledMap);
            next_state.set(MapState::Loaded);
        } else {
            warn!("Cannot respawn: no map loaded ?");
        }
    }
}

fn handle_unload(
    mut commands: Commands,
    mut maps: ResMut<Assets<TiledMap>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    maps_query: Query<Entity, With<TiledMapMarker>>,
    mut next_state: ResMut<NextState<MapState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyU) {
        // This example shows that the map gets properly unloaded if the
        // `TiledMap` asset is removed.
        //
        // However, typically you would remove the map entity instead.
        info!("Unload map");
        let handles: Vec<_> = maps.iter().map(|(handle, _)| handle).collect();
        for handle in handles {
            // This will cause the map to unload.
            maps.remove(handle);
        }
        next_state.set(MapState::Unloaded);
    } else if keyboard_input.just_pressed(KeyCode::KeyI) {
        // Just remove the entities directly. This will also unload the map.
        info!("Remove map entities");
        for entity in maps_query.iter() {
            commands.entity(entity).despawn_recursive();
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
