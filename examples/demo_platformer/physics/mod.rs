use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub mod movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default(),
        PhysicsPlugins::default(),
    ));
    app.add_plugins(movement::plugin);
    app.insert_resource(Gravity(Vec2::NEG_Y * 10000.));
}
