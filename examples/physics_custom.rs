//! This example shows how to use a custom physics backend.

use bevy::{
    color::palettes::css::{PURPLE, RED},
    ecs::component::StorageType,
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
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
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TiledMapHandle(asset_server.load("finite.tmx")));
}

#[derive(Default)]
struct MyCustomPhysicsBackend;

// This simple example will just spawn an entity with a `MyCustomPhysicsComponent` Component,
// at the center of where the Tiled collider is.
impl TiledPhysicsBackend for MyCustomPhysicsBackend {
    fn spawn_collider(
        &self,
        commands: &mut Commands,
        _map: &Map,
        collider_source: &TiledColliderSource,
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

        let color = match collider_source {
            TiledColliderSource::Object {
                layer_id: _,
                object_id: _,
            } => Color::from(PURPLE),
            TiledColliderSource::Tile {
                layer_id: _,
                x: _,
                y: _,
            } => Color::from(RED),
        };

        let entity = commands.spawn(MyCustomPhysicsComponent(color)).id();

        Some((pos, entity))
    }
}

// For debugging purpose, we will also add a 2D mesh where the collider is.
struct MyCustomPhysicsComponent(pub Color);

impl Component for MyCustomPhysicsComponent {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, entity, _| {
            let color = world.get::<MyCustomPhysicsComponent>(entity).unwrap().0;
            let mesh = world
                .resource_mut::<Assets<Mesh>>()
                .add(Rectangle::from_length(10.));
            let material = world.resource_mut::<Assets<ColorMaterial>>().add(color);
            world
                .commands()
                .entity(entity)
                .insert(MaterialMesh2dBundle {
                    mesh: mesh.into(),
                    transform: Transform::default(),
                    material,
                    ..default()
                });
        });
    }
}
