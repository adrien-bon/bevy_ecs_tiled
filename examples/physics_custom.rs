//! This example shows how to use a custom physics backend.

use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::{PURPLE, RED},
    prelude::*,
};
use bevy_ecs_tiled::{physics::backend::TiledPhysicsBackendOutput, prelude::*};

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
/// Implements the [`TiledPhysicsBackend`] trait. Instead of spawning real physics colliders,
/// this backend draws a mesh outlining the polygons for each collider, using a custom color
/// depending on the collider type (object or tile layer).
#[derive(Default, Debug, Clone, Reflect)]
#[reflect(Default, Debug)]
struct MyCustomPhysicsBackend;

impl TiledPhysicsBackend for MyCustomPhysicsBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        source: &TiledEvent<ColliderCreated>,
        multi_polygon: MultiPolygon<f32>,
    ) -> Vec<TiledPhysicsBackendOutput> {
        let (name, color) = match source.event.0 {
            TiledCollider::Object => (String::from("Custom[Object]"), Color::from(PURPLE)),
            TiledCollider::TilesLayer => (String::from("Custom[TilesLayer]"), Color::from(RED)),
        };

        vec![TiledPhysicsBackendOutput {
            name: name.clone(),
            entity: commands
                // In this specific case we want to draw a mesh which require access
                // to `Assets<Mesh>` and `Assets<ColorMaterial>` resources.
                // We'll wrap everything in a custom command to get access to `World`
                // so we can retrieve these resources.
                .spawn_empty()
                .queue(CustomColliderCommand {
                    color,
                    multi_polygon: multi_polygon.clone(),
                })
                .id(),
            transform: Transform::default(),
        }]
    }
}

// Custom command implementation: nothing fancy here,
// we just store the polygons and color to use for the mesh.
struct CustomColliderCommand {
    multi_polygon: MultiPolygon<f32>,
    color: Color,
}

impl EntityCommand for CustomColliderCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let mut vertices = vec![];
        multi_polygon_as_line_strings(&self.multi_polygon)
            .into_iter()
            .for_each(|ls| {
                ls.lines().for_each(|l| {
                    let points = l.points();
                    vertices.push([points.0.x(), points.0.y(), 10.]);
                    vertices.push([points.1.x(), points.1.y(), 10.]);
                });
            });

        let mut mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::LineList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

        let mesh_handle = entity.resource_mut::<Assets<Mesh>>().add(mesh);
        let material_handle = entity
            .resource_mut::<Assets<ColorMaterial>>()
            .add(self.color);
        entity.insert((Mesh2d(mesh_handle), MeshMaterial2d(material_handle)));
    }
}
