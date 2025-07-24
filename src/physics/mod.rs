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
use collider::spawn_colliders;

/// Physics plugin.
///
/// Must be added to your app in order to automatically spawn physics colliders using the provided [`TiledPhysicsBackend`].
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// // Using Avian backend for demonstration purpose, note that we also support TiledPhysicsRapierBackend
/// App::new()
///     .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default());
/// ```
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
    mut layer_event: EventReader<TiledEvent<LayerCreated>>,
    mut commands: Commands,
    assets: Res<Assets<TiledMapAsset>>,
    maps_query: Query<(&TiledPhysicsSettings<T>, &TilemapAnchor), With<TiledMap>>,
    mut event_writer: EventWriter<TiledEvent<ColliderCreated>>,
) {
    for ev in layer_event.read() {
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

        if TiledNameFilter::from(&settings.tiles_layer_filter).contains(&layer.name) {
            spawn_colliders::<T>(
                &settings.backend,
                &mut commands,
                &assets,
                anchor,
                &settings.tiles_objects_filter,
                ev.transmute(None, ColliderCreated(TiledCollider::TilesLayer)),
                ev.origin,
                &mut event_writer,
            );
        }
    }
}

fn collider_from_object<T: TiledPhysicsBackend>(
    mut object_event: EventReader<TiledEvent<ObjectCreated>>,
    mut commands: Commands,
    assets: Res<Assets<TiledMapAsset>>,
    maps_query: Query<(&TiledPhysicsSettings<T>, &TilemapAnchor), With<TiledMap>>,
    mut event_writer: EventWriter<TiledEvent<ColliderCreated>>,
) {
    for ev in object_event.read() {
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

        if TiledNameFilter::from(&settings.objects_layer_filter).contains(&layer.name)
            && TiledNameFilter::from(&settings.objects_filter).contains(&object.name)
        {
            spawn_colliders::<T>(
                &settings.backend,
                &mut commands,
                &assets,
                anchor,
                match object.get_tile() {
                    Some(_) => &settings.tiles_objects_filter,
                    None => &TiledName::All,
                },
                ev.transmute(None, ColliderCreated(TiledCollider::Object)),
                ev.origin,
                &mut event_writer,
            );
        }
    }
}
