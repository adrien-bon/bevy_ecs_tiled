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
        ))
        .observe(
            |t: Trigger<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                commands.entity(t.event().origin).insert(RigidBody::Static);
            },
        );
}
