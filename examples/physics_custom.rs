//! This example shows how to use a custom physics backend.

use bevy::{
    color::palettes::css::{PURPLE, RED},
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
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
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(TiledMapHandle(asset_server.load("finite.tmx")));
}

#[derive(Default, Clone, Reflect)]
struct MyCustomPhysicsBackend;

// This simple example will just spawn an entity with a `MyCustomPhysicsComponent` Component,
// at the center of where the Tiled collider is.
impl TiledPhysicsBackend for MyCustomPhysicsBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        _map: &Map,
        _filter: &TiledNameFilter,
        collider: &TiledCollider,
    ) -> Vec<TiledColliderSpawnInfos> {
        match collider {
            TiledCollider::Object {
                layer_id: _,
                object_id: _,
            } => {
                vec![TiledColliderSpawnInfos {
                    name: "Custom[Object]".to_string(),
                    entity: commands
                        .spawn(MyCustomPhysicsComponent(Color::from(PURPLE)))
                        .id(),
                    transform: Transform::default(),
                }]
            }
            TiledCollider::TilesLayer { layer_id: _ } => {
                vec![TiledColliderSpawnInfos {
                    name: "Custom[TilesLayer]".to_string(),
                    entity: commands
                        .spawn(MyCustomPhysicsComponent(Color::from(RED)))
                        .id(),
                    transform: Transform::default(),
                }]
            }
        }
    }
}

// For debugging purpose, we will also add a 2D mesh where the collider is.
#[derive(Component)]
#[component(on_add = on_physics_component_added)]
struct MyCustomPhysicsComponent(pub Color);

fn on_physics_component_added(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    let color = world.get::<MyCustomPhysicsComponent>(entity).unwrap().0;
    let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(Rectangle::from_length(10.));
    let material = world.resource_mut::<Assets<ColorMaterial>>().add(color);
    world
        .commands()
        .entity(entity)
        .insert((Mesh2d(mesh), MeshMaterial2d(material)));
}
