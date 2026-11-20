//! Load the Tiled map and configure how physics colliders are created for it.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, startup);
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            TiledMap(asset_server.load("demo_platformer/demo.tmx")),
            TilemapAnchor::Center,
            TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
                // Only add colliders for the objects we use as trigger zones:
                // we do not want the patrol routes to become walls.
                objects_filter: TiledFilter::from(["KillZone", "TeleportZone"]),
                // Split tiles colliders in chunks, so only a small part of them
                // would have to be computed again if we were to update some tiles.
                tiles_chunk_size: TiledColliderChunkSize::Size(UVec2::splat(16)),
                ..default()
            },
        ))
        // Colliders created from the map never move: make them static
        // A `ColliderCreated` event is sent for each collider. A tiles layer or an object can have several
        // colliders (eg. `TiledColliderStrategy::LineStrip` or collider chunks), held by its entity or by its
        // descendants (the event `origin`).
        // Insert the rigid body on the layer or object itself: colliders are attached to the rigid body of
        // their closest ancestor, so they all belong to a single body. Inserting it on each `origin` instead
        // would create several nested rigid bodies, which only behaves correctly as long as they never move.
        .observe(
            |collider_created: On<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                commands
                    .entity(collider_created.source_entity())
                    .insert(RigidBody::Static);
            },
        );
}
