//! This module handles all things related to physics.
//!
//! It is only available when the `physics` feature is enabled.
//!
//! See the [dedicated book section](https://adrien-bon.github.io/bevy_ecs_tiled/guides/physics.html) for more information.

pub mod collider;

#[cfg(feature = "rapier")]
pub mod rapier;

#[cfg(feature = "avian")]
pub mod avian;

use crate::prelude::*;
use bevy::{prelude::*, reflect::Reflectable};
use prelude::*;

/// `bevy_ecs_tiled` physics public exports.
pub mod prelude {
    #[cfg(feature = "avian")]
    pub use super::avian::*;
    pub use super::collider::*;
    #[cfg(feature = "rapier")]
    pub use super::rapier::*;
    pub use super::TiledPhysicsBackend;
    pub use super::TiledPhysicsPlugin;
    pub use super::TiledPhysicsSettings;
}

/// Physics backend public trait.
///
/// A custom physics backend should implement this trait.
pub trait TiledPhysicsBackend:
    Default + Clone + 'static + std::marker::Sync + std::marker::Send + FromReflect + Reflectable
{
    /// Function responsible for spawning a physics collider
    ///
    /// This function should spawn an [Entity] representing a single physics
    /// collider and return informations about it.
    /// In case the provided [TiledColliderSource] is not supported, it should
    /// not spawn anything and return `None`.
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        tiled_map: &TiledMap,
        filter: &TiledNameFilter,
        collider: &TiledCollider,
    ) -> Vec<TiledColliderSpawnInfos>;
}

/// Physics related settings.
#[derive(Clone, Component, Default, Reflect)]
pub struct TiledPhysicsSettings<T: TiledPhysicsBackend> {
    /// Specify which Tiled object to add colliders for using their layer name.
    ///
    /// Colliders will be automatically added for all objects whose containing layer name matches this filter.
    /// By default, we add colliders for all objects.
    pub objects_layer_filter: TiledName,
    /// Specify which Tiled object to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all objects whose name matches this filter.
    /// By default, we add colliders for all objects.
    pub objects_filter: TiledName,
    /// Specify which tiles collision object to add colliders for using their layer name.
    ///
    /// Colliders will be automatically added for all tiles collision objects whose layer name matches this filter.
    /// By default, we add colliders for all collision objects.
    pub tiles_layer_filter: TiledName,
    /// Specify which tiles collision object to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all tiles collision objects whose name matches this filter.
    /// By default, we add colliders for all collision objects.
    pub tiles_objects_filter: TiledName,
    /// Physics backend to use for adding colliders.
    pub backend: T,
}

/// Physics plugin.
///
/// Must be added to your app in order to automatically spawn physics colliders using the provided [TiledPhysicsBackend].
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
#[derive(Default)]
pub struct TiledPhysicsPlugin<T: TiledPhysicsBackend>(std::marker::PhantomData<T>);

impl<T: TiledPhysicsBackend> Plugin for TiledPhysicsPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PreUpdate,
            (
                collider_from_tiles_layer::<T>.after(crate::map::process_loaded_maps),
                collider_from_object::<T>.after(crate::map::process_loaded_maps),
            ),
        )
        .register_type::<TiledPhysicsSettings<T>>();
    }
}

#[allow(clippy::type_complexity)]
fn get_physics_settings<T: TiledPhysicsBackend>(
    map_entity: Entity,
    q_maps: &Query<(Option<&Parent>, Option<&TiledPhysicsSettings<T>>), With<TiledMapMarker>>,
    q_worlds: &Query<Option<&TiledPhysicsSettings<T>>, With<TiledWorldMarker>>,
) -> TiledPhysicsSettings<T> {
    q_maps
        .get(map_entity)
        .ok()
        .and_then(|(parent, settings)| {
            if settings.is_some() {
                // Try to use physics settings from the map
                settings.cloned()
            } else {
                // Fallback on physics settings from the parent world
                parent
                    .and_then(|p| q_worlds.get(p.get()).ok())
                    .and_then(|s| s.cloned())
            }
        })
        .unwrap_or_default()
}

#[allow(clippy::type_complexity)]
fn collider_from_tiles_layer<T: TiledPhysicsBackend>(
    mut layer_event: EventReader<TiledLayerCreated>,
    mut commands: Commands,
    map_asset: Res<Assets<TiledMap>>,
    q_maps: Query<(Option<&Parent>, Option<&TiledPhysicsSettings<T>>), With<TiledMapMarker>>,
    q_worlds: Query<Option<&TiledPhysicsSettings<T>>, With<TiledWorldMarker>>,
) {
    for ev in layer_event.read() {
        let settings = get_physics_settings(ev.map.entity, &q_maps, &q_worlds);
        let Some(tiled_map) = ev.map.get_map_asset(&map_asset) else {
            return;
        };
        let Some(layer) = ev.get_layer(&map_asset) else {
            return;
        };
        let tiled::LayerType::Tiles(_) = layer.layer_type() else {
            return;
        };

        if TiledNameFilter::from(&settings.tiles_layer_filter).contains(&layer.name) {
            collider::spawn_colliders(
                &settings.backend,
                ev.entity,
                &mut commands,
                tiled_map,
                &settings.tiles_objects_filter,
                &TiledCollider::from_tiles_layer(ev.id),
            );
        }
    }
}

#[allow(clippy::type_complexity)]
fn collider_from_object<T: TiledPhysicsBackend>(
    mut object_event: EventReader<TiledObjectCreated>,
    mut commands: Commands,
    map_asset: Res<Assets<TiledMap>>,
    q_maps: Query<(Option<&Parent>, Option<&TiledPhysicsSettings<T>>), With<TiledMapMarker>>,
    q_worlds: Query<Option<&TiledPhysicsSettings<T>>, With<TiledWorldMarker>>,
) {
    for ev in object_event.read() {
        let settings = get_physics_settings(ev.layer.map.entity, &q_maps, &q_worlds);
        let Some(tiled_map) = ev.layer.map.get_map_asset(&map_asset) else {
            return;
        };
        let Some(layer) = ev.layer.get_layer(&map_asset) else {
            return;
        };
        let Some(object) = ev.get_object(&map_asset) else {
            return;
        };

        if TiledNameFilter::from(&settings.objects_layer_filter).contains(&layer.name)
            && TiledNameFilter::from(&settings.objects_filter).contains(&object.name)
        {
            collider::spawn_colliders(
                &settings.backend,
                ev.entity,
                &mut commands,
                tiled_map,
                match object.get_tile() {
                    Some(_) => &settings.tiles_objects_filter,
                    None => &TiledName::All,
                },
                &TiledCollider::from_object(ev.layer.id, ev.id),
            );
        }
    }
}
