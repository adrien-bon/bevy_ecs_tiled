use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy::render::view::RenderLayers;

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
        )
        .observe(
            |trigger: Trigger<TiledEvent<LayerCreated>>,
             layers_query: Query<(Entity, &TiledLayer)>,
             children_query: Query<&Children>,
             mut commands: Commands| {
                let Ok((layer_entity, layer)) = layers_query.get(trigger.event().origin) else {
                    return;
                };
                if !matches!(layer, TiledLayer::Tiles) {
                    return;
                }
                for e in std::iter::once(layer_entity).chain(children_query.iter_descendants(layer_entity)) {
                    commands
                        .entity(e)
                        .insert(RenderLayers::from_layers(&[0, 1])); // Also appear on minimap
                }
            },
        );
}
