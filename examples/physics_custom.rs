//! This example shows how to use a custom physics backend.

use bevy::{
    color::palettes::css::{PURPLE, RED},
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
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
        // bevy_ecs_tiled physics plugin: this is where we select which physics backend to use
        // Here we use a custom backend (see below)
        .add_plugins(TiledPhysicsPlugin::<MyCustomPhysicsBackend>::default())
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(TiledMapHandle(
        asset_server.load("maps/orthogonal/finite.tmx"),
    ));
}

#[derive(Default, Clone, Reflect)]
struct MyCustomPhysicsBackend;

// This simple example will just spawn an entity with a `MyCustomPhysicsComponent` Component,
// at the center of where the Tiled collider is.
impl TiledPhysicsBackend for MyCustomPhysicsBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        _tiled_map: &TiledMap,
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
