//! This module handles all things related to physics.
//!
//! It is only available when the `physics` feature is enabled.
//!
//! See the [dedicated book section](https://adrien-bon.github.io/bevy_ecs_tiled/guides/physics.html) for more information.

pub mod backend;
pub mod collider;
pub mod settings;

use crate::prelude::*;
use bevy::prelude::*;

/// Physics plugin.
///
/// Must be added to your app in order to automatically spawn physics colliders using the provided [`TiledPhysicsBackend`].
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// // Using Avian backend for demonstration purpose,
/// // note that we also support TiledPhysicsRapierBackend
/// App::new()
///     .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default());
/// ```
///
/// We provide several [`TiledPhysicsBackend`] that can be used out of the box:
/// - [`TiledPhysicsAvianBackend`]: for the Avian 2D physics engine
// /// - [`TiledPhysicsRapierBackend`]: for the Rapier 2D physics engine
///
#[derive(Default, Copy, Clone, Debug)]
pub struct TiledPhysicsPlugin<T: TiledPhysicsBackend>(std::marker::PhantomData<T>);

impl<T: TiledPhysicsBackend> Plugin for TiledPhysicsPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<T>();
        app.add_systems(
            PreUpdate,
            (collider_from_tiles_layer::<T>, collider_from_object::<T>)
                .chain()
                .in_set(TiledPreUpdateSystems::SpawnPhysicsColliders),
        );
        app.add_plugins((backend::plugin, collider::plugin, settings::plugin::<T>));
    }
}

fn collider_from_tiles_layer<T: TiledPhysicsBackend>(
    mut layer_created: MessageReader<TiledEvent<LayerCreated>>,
    mut commands: Commands,
    assets: Res<Assets<TiledMapAsset>>,
    maps_query: Query<(&TiledPhysicsSettings<T>, &TilemapAnchor), With<TiledMap>>,
    mut message_writer: MessageWriter<TiledEvent<ColliderCreated>>,
) {
    for ev in layer_created.read() {
        let (settings, anchor) = ev
            .get_map_entity()
            .and_then(|e| maps_query.get(e).ok())
            .expect("TiledPhysicsSettings<T> component should be on map entity");

        let Some(layer) = ev.get_layer(&assets) else {
            continue;
        };

        let tiled::LayerType::Tiles(_) = layer.layer_type() else {
            continue;
        };

        if settings.tiles_layer_filter.matches(&layer.name) {
            collider::spawn_colliders::<T>(
                &settings.backend,
                &mut commands,
                &assets,
                anchor,
                &settings.tiles_objects_filter,
                ev.transmute(
                    None,
                    ColliderCreated::new(TiledColliderSource::TilesLayer, ev.origin),
                ),
                &mut message_writer,
            );
        }
    }
}

fn collider_from_object<T: TiledPhysicsBackend>(
    mut object_created: MessageReader<TiledEvent<ObjectCreated>>,
    mut commands: Commands,
    assets: Res<Assets<TiledMapAsset>>,
    maps_query: Query<(&TiledPhysicsSettings<T>, &TilemapAnchor), With<TiledMap>>,
    mut message_writer: MessageWriter<TiledEvent<ColliderCreated>>,
) {
    for ev in object_created.read() {
        let (settings, anchor) = ev
            .get_map_entity()
            .and_then(|e| maps_query.get(e).ok())
            .expect("TiledPhysicsSettings<T> component should be on map entity");

        let Some(layer) = ev.get_layer(&assets) else {
            continue;
        };

        let Some(object) = ev.get_object(&assets) else {
            continue;
        };

        if settings.objects_layer_filter.matches(&layer.name)
            && settings.objects_filter.matches(&object.name)
        {
            collider::spawn_colliders::<T>(
                &settings.backend,
                &mut commands,
                &assets,
                anchor,
                match object.get_tile() {
                    Some(_) => &settings.tiles_objects_filter,
                    None => &TiledFilter::All,
                },
                ev.transmute(
                    None,
                    ColliderCreated::new(TiledColliderSource::Object, ev.origin),
                ),
                &mut message_writer,
            );
        }
    }
}
