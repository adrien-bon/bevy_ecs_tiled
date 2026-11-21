//! This example shows a simple player-controlled object using Rapier physics. You can move the object using arrow keys.
//!
//! Map colliders are static, except for the objects of the "Dynamic objects" layer which are turned
//! into dynamic bodies: they fall and can be pushed around.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_rapier2d::prelude::*;

mod helper;

const MOVE_SPEED: f32 = 200.;
const GRAVITY_SCALE: f32 = 10.0;

/// Name of the Tiled objects layer whose objects become dynamic bodies
const DYNAMIC_OBJECTS_LAYER: &str = "Dynamic objects";

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
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsRapierBackend>::default())
        // Rapier physics plugins
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, move_player)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Text(String::from(
        "Move the ball using arrow keys to push the objects around or try to rotate the map!",
    )));
    commands
        .spawn((
            TiledMap(asset_server.load("maps/orthogonal/multiple_layers_with_colliders.tmx")),
            TilemapAnchor::Center,
        ))
        // Wait for map loading to complete and spawn a simple player-controlled object
        .observe(|_: On<TiledEvent<MapCreated>>, mut commands: Commands| {
            commands.spawn((
                RigidBody::Dynamic,
                PlayerMarker,
                Name::new("PlayerControlledObject (Rapier physics)"),
                Collider::ball(10.),
                Velocity::zero(),
                GravityScale(GRAVITY_SCALE),
                Transform::from_xyz(50., -50., 0.),
            ));
        })
        // Objects of the "Dynamic objects" layer become dynamic bodies: they need colliders
        // with an actual area to compute their mass, so we triangulate them instead of using
        // the default `TiledColliderStrategy::Polyline`, which only creates their outline.
        // Inserting this component on the layer applies to all its objects.
        .observe(
            |layer_created: On<TiledEvent<LayerCreated>>,
             layers_query: Query<&TiledName>,
             mut commands: Commands| {
                let layer = layer_created.origin;
                if is_dynamic_layer(&layers_query, Some(layer)) {
                    commands
                        .entity(layer)
                        .insert(TiledColliderStrategy::Triangulation);
                }
            },
        )
        // Insert a rigid body for all the map colliders: `RigidBody::Dynamic` for the objects of the
        // "Dynamic objects" layer, `RigidBody::Fixed` for everything else.
        // A `ColliderCreated` event is sent for each collider. A tiles layer or an object can have several
        // colliders (eg. `TiledColliderStrategy::LineStrip` or collider chunks), held by its entity or by its
        // descendants (the event `origin`).
        // Insert the rigid body on the layer or object itself: colliders are attached to the rigid body of
        // their closest ancestor, so they all belong to a single body. Inserting it on each `origin` instead
        // would create several nested rigid bodies, which only behaves correctly as long as they never move.
        .observe(
            |collider_created: On<TiledEvent<ColliderCreated>>,
             layers_query: Query<&TiledName>,
             mut commands: Commands| {
                let layer = collider_created.layer_entity();
                let rigid_body = if is_dynamic_layer(&layers_query, layer) {
                    RigidBody::Dynamic
                } else {
                    RigidBody::Fixed
                };
                commands
                    .entity(collider_created.source_entity())
                    .insert(rigid_body);
            },
        );
}

/// Whether the given layer is the one holding our dynamic objects
fn is_dynamic_layer(layers_query: &Query<&TiledName>, layer: Option<Entity>) -> bool {
    layer
        .and_then(|layer| layers_query.get(layer).ok())
        .is_some_and(|name| name.0 == DYNAMIC_OBJECTS_LAYER)
}

// A 'player' marker component
#[derive(Default, Clone, Component)]
pub struct PlayerMarker;

// A simplistic controller
fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut Velocity, With<PlayerMarker>>,
) {
    for mut rb_vels in player.iter_mut() {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction += Vec2::new(1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction -= Vec2::new(1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction += Vec2::new(0.0, 1.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction -= Vec2::new(0.0, 1.0);
        }

        if direction != Vec2::ZERO {
            direction /= direction.length();
        }

        rb_vels.linear = direction * MOVE_SPEED;
    }
}
