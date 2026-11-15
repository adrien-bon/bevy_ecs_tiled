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
                objects_filter: TiledFilter::from(vec!["KillZone", "TeleportZone"]),
                // Split tiles colliders in chunks, so only a small part of them
                // would have to be computed again if we were to update some tiles.
                tiles_chunk_size: Some(UVec2::splat(16)),
                ..default()
            },
        ))
        // Colliders created from the map never move
        .observe(
            |collider_created: On<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                commands
                    .entity(collider_created.event().origin)
                    .insert(RigidBody::Static);
            },
        );
}
