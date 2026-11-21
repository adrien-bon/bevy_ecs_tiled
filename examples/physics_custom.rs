//! This example shows how to use a custom physics backend.

use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::{PURPLE, RED},
    ecs::{lifecycle::HookContext, world::DeferredWorld},
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
        .add_plugins(TiledPlugin::default())
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
    // Just spawn a 2D camera and a Tiled map
    commands.spawn(Camera2d);
    commands.spawn(TiledMap(asset_server.load("maps/orthogonal/finite.tmx")));
}

/// Custom physics backend for demonstration purposes.
///
/// Implements the [`TiledPhysicsBackend`] trait. Instead of creating real physics colliders,
/// this backend produces a [`MyCustomCollider`] component for each collider geometry.
/// A mesh outlining this geometry is then drawn, using a custom color depending
/// on the collider type (object or tile layer).
#[derive(Default, Debug, Clone, Reflect)]
#[reflect(Default, Debug)]
struct MyCustomPhysicsBackend;

impl TiledPhysicsBackend for MyCustomPhysicsBackend {
    // The components our backend produces for each collider: `bevy_ecs_tiled` takes care
    // of inserting them on the right entity, and of removing them when the collider
    // source (a tiles layer or an object) is modified.
    type Collider = MyCustomCollider;

    fn colliders(
        &self,
        // We always draw the geometry outline, whatever the requested strategy
        _strategy: TiledColliderStrategy,
        source: TiledColliderSource,
        geometry: &TiledColliderGeometry,
    ) -> Vec<Self::Collider> {
        let color = match source {
            TiledColliderSource::Object => Color::from(PURPLE),
            TiledColliderSource::TilesLayer => Color::from(RED),
        };
        let mut vertices = vec![];
        // The outline does not include the edges shared with adjacent collider chunks, if any
        for line_string in &geometry.outline() {
            for line in line_string.lines() {
                let points = line.points();
                vertices.push([points.0.x(), points.0.y(), 10.]);
                vertices.push([points.1.x(), points.1.y(), 10.]);
            }
        }
        vec![MyCustomCollider { vertices, color }]
    }
}

/// Our custom "collider": the outline of the collider geometry and the color to draw it.
///
/// We want to draw a mesh, which requires access to `Assets<Mesh>` and `Assets<ColorMaterial>`
/// resources: this is done in component hooks, which also remove the mesh along with the collider.
#[derive(Component)]
#[component(on_insert = on_collider_inserted, on_remove = on_collider_removed)]
struct MyCustomCollider {
    vertices: Vec<[f32; 3]>,
    color: Color,
}

fn on_collider_inserted(mut world: DeferredWorld, context: HookContext) {
    let Some(collider) = world.get::<MyCustomCollider>(context.entity) else {
        return;
    };
    let (vertices, color) = (collider.vertices.clone(), collider.color);

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::LineList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    let mesh_handle = world.resource_mut::<Assets<Mesh>>().add(mesh);
    let material_handle = world.resource_mut::<Assets<ColorMaterial>>().add(color);
    world
        .commands()
        .entity(context.entity)
        .insert((Mesh2d(mesh_handle), MeshMaterial2d(material_handle)));
}

fn on_collider_removed(mut world: DeferredWorld, context: HookContext) {
    world
        .commands()
        .entity(context.entity)
        .try_remove::<(Mesh2d, MeshMaterial2d<ColorMaterial>)>();
}
