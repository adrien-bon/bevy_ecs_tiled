//! This example shows how to use physics events.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::Map;

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
        // bevy_ecs_tiled physics plugin: this is where we select which physics backend to use
        .add_plugins(TiledPhysicsPlugin::<MyCustomPhysicsBackend>::default())
        // Add observers for physics collider events
        .add_observer(handle_physics_events)
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(TiledMapHandle(asset_server.load("finite.tmx")));
}

// Here we can handle our physics events: we will receive one event per `TiledColliderSource`
// It means on event per object and one event per tile with collision object (even if tile has multiple collision objects)
fn handle_physics_events(trigger: Trigger<TiledColliderCreated>) {
    match trigger.event().collider_source.ty {
        TiledColliderSourceType::Object {
            layer_id,
            object_id,
        } => {
            info!(
                "Created a collider for object (layer={}, ID={}): {:?}",
                layer_id,
                object_id,
                trigger.event(),
            );
        }
        TiledColliderSourceType::TilesLayer { layer_id } => {
            info!(
                "Created a collider for tiles layer (layer={}): {:?}",
                layer_id,
                trigger.event(),
            );
        }
    }
}

// Here goes the custom physics backend definition, which is not related to current example
// physics events are always sent, whatever the backend is.

#[derive(Component)]
struct MyCustomPhysicsComponent;

#[derive(Default, Clone, Reflect)]
struct MyCustomPhysicsBackend;

// This simple example will just spawn an entity with a `MyCustomPhysicsComponent` Component,
// at the center of where the Tiled collider is.
impl TiledPhysicsBackend for MyCustomPhysicsBackend {
    fn spawn_collider(
        &self,
        commands: &mut Commands,
        _map: &Map,
        collider_source: &TiledColliderSource,
    ) -> Vec<TiledColliderSpawnInfos> {
        match collider_source.ty {
            TiledColliderSourceType::Object {
                layer_id: _,
                object_id: _,
            } => {
                vec![TiledColliderSpawnInfos {
                    name: "Custom[Object]".to_string(),
                    entity: commands.spawn(MyCustomPhysicsComponent).id(),
                    position: Vec2::ZERO,
                    rotation: 0.,
                }]
            }
            TiledColliderSourceType::TilesLayer { layer_id: _ } => {
                vec![TiledColliderSpawnInfos {
                    name: "Custom[TilesLayer]".to_string(),
                    entity: commands.spawn(MyCustomPhysicsComponent).id(),
                    position: Vec2::ZERO,
                    rotation: 0.,
                }]
            }
        }
    }
}
