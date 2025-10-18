use avian2d::prelude::*;
use bevy::{camera::visibility::RenderLayers, prelude::*};
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
            |collider_created: On<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                commands
                    .entity(*collider_created.event().event.collider_of)
                    .insert(RigidBody::Static);
            },
        )
        .observe(
            |layer_created: On<TiledEvent<LayerCreated>>,
             layers_query: Query<(Entity, &TiledLayer)>,
             children_query: Query<&Children>,
             mut commands: Commands| {
                let Ok((layer_entity, layer)) = layers_query.get(layer_created.event().origin) else {
                    return;
                };
                if !matches!(layer, TiledLayer::Tiles) {
                    return;
                }
                for e in std::iter::once(layer_entity)
                    .chain(children_query.iter_descendants(layer_entity))
                {
                    commands
                        .entity(e)
                        .insert(RenderLayers::from_layers(&[0, 1])); // Also appear on minimap
                }
            },
        );
}
