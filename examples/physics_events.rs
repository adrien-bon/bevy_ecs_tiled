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
        TiledColliderSourceType::Tile {
            layer_id,
            x,
            y,
            object_id,
        } => {
            info!(
                "Created a collider for tile (layer={}, x={}, y={}, object_id={}): {:?}",
                layer_id,
                x,
                y,
                object_id,
                trigger.event(),
            );
        }
    }
}

// Here goes the custom physics backend definition, which is not related to current example
// physics events are always sent, whatever the backend is.

#[derive(Component)]
struct MyCustomPhysicsComponent;

#[derive(Default, Clone)]
struct MyCustomPhysicsBackend;

// This simple example will just spawn an entity with a `MyCustomPhysicsComponent` Component,
// at the center of where the Tiled collider is.
impl TiledPhysicsBackend for MyCustomPhysicsBackend {
    fn spawn_collider(
        &self,
        commands: &mut Commands,
        map: &Map,
        collider_source: &TiledColliderSource,
    ) -> Option<TiledColliderSpawnInfos> {
        // TODO: use this function once I figure out how to prevent cloning ObjectData
        // let object_data = collider_source.object_data(map)?;

        let tile = collider_source.tile(map);
        let object = collider_source.object(map);

        let object_data = (match collider_source.ty {
            TiledColliderSourceType::Tile {
                layer_id: _,
                x: _,
                y: _,
                object_id,
            } => tile
                .as_ref()
                .and_then(|tile| tile.collision.as_ref())
                .map(|collision| collision.object_data())
                .and_then(|objects| objects.get(object_id)),
            TiledColliderSourceType::Object {
                layer_id: _,
                object_id: _,
            } => object.as_deref(),
        })?;

        let pos = match &object_data.shape {
            tiled::ObjectShape::Rect { width, height } => Vec2::new(width / 2., -height / 2.),
            tiled::ObjectShape::Ellipse { width, height } => Vec2::new(width / 2., -height / 2.),
            tiled::ObjectShape::Polyline { points: _ } => Vec2::ZERO,
            tiled::ObjectShape::Polygon { points: _ } => Vec2::ZERO,
            _ => {
                return None;
            }
        };

        Some(TiledColliderSpawnInfos {
            name: format!("Custom[{}]", object_data.name),
            entity: commands.spawn(MyCustomPhysicsComponent).id(),
            position: pos,
            rotation: -object_data.rotation,
        })
    }
}
