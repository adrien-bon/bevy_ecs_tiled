//! This example shows how to use physics events.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::{Map, ObjectData};

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
        .observe(handle_physics_events)
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TiledMapHandle(asset_server.load("finite.tmx")));
}

// Here we can handle our physics events: we will receive one event per `TiledColliderSource`
// It means on event per object and one event per tile with collision object (even if tile has multiple collision objects)
fn handle_physics_events(trigger: Trigger<TiledColliderCreated>) {
    match trigger.event().collider_source {
        TiledColliderSource::Object {
            layer_id,
            object_id,
        } => {
            info!(
                "Created {} collider(s) for object (layer={}, ID={})",
                trigger.event().colliders_entities_list.len(),
                layer_id,
                object_id
            );
        }
        TiledColliderSource::Tile { layer_id, x, y } => {
            info!(
                "Created {} collider(s) for tile (layer={}, x={}, y={})",
                trigger.event().colliders_entities_list.len(),
                layer_id,
                x,
                y
            );
        }
    }
}

// Here goes the custom physics backend definition, which is not related to current example
// physics events are always sent, whatever the backend is.

#[derive(Component)]
struct MyCustomPhysicsComponent;

#[derive(Default)]
struct MyCustomPhysicsBackend;

// This simple example will just spawn an entity with a `MyCustomPhysicsComponent` Component,
// at the center of where the Tiled collider is.
impl TiledPhysicsBackend for MyCustomPhysicsBackend {
    fn spawn_collider(
        &self,
        commands: &mut Commands,
        _map: &Map,
        _collider_source: &TiledColliderSource,
        object_data: &ObjectData,
    ) -> Option<(Vec2, Entity)> {
        let pos = match &object_data.shape {
            tiled::ObjectShape::Rect { width, height } => Vec2::new(width / 2., -height / 2.),
            tiled::ObjectShape::Ellipse { width, height } => Vec2::new(width / 2., -height / 2.),
            tiled::ObjectShape::Polyline { points: _ } => Vec2::ZERO,
            tiled::ObjectShape::Polygon { points: _ } => Vec2::ZERO,
            _ => {
                return None;
            }
        };

        let entity = commands.spawn(MyCustomPhysicsComponent).id();
        Some((pos, entity))
    }
}
