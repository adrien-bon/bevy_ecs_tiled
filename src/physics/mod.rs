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
use bevy::prelude::*;
use prelude::*;
use tiled::Map;

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
    Default + Clone + 'static + std::marker::Sync + std::marker::Send
{
    /// Function responsible for spawning a physics collider
    ///
    /// This function should spawn an [Entity] representing a single physics
    /// collider and return informations about it.
    /// In case the provided [TiledColliderSource] is not supported, it should
    /// not spawn anything and return `None`.
    fn spawn_collider(
        &self,
        commands: &mut Commands,
        map: &Map,
        collider_source: &TiledColliderSource,
    ) -> Vec<TiledColliderSpawnInfos>;
}

/// Physics related settings.
#[derive(Clone, Component, Default)]
pub struct TiledPhysicsSettings<T: TiledPhysicsBackend> {
    /// Specify which Tiled object to add colliders for using their layer name.
    ///
    /// Colliders will be automatically added for all objects whose containing layer name matches this filter.
    /// By default, we add colliders for all objects.
    pub objects_layer_filter: ObjectNames,
    /// Specify which Tiled object to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all objects whose name matches this filter.
    /// By default, we add colliders for all objects.
    pub objects_filter: ObjectNames,
    /// Specify which tiles collision object to add colliders for using their layer name.
    ///
    /// Colliders will be automatically added for all tiles collision objects whose layer name matches this filter.
    /// By default, we add colliders for all collision objects.
    pub tiles_layer_filter: ObjectNames,
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
/// // Using Avian backend for demonstrationg purpose, note that we also support TiledPhysicsRapierBackend
/// App::new()
///     .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default());
/// ```
#[derive(Default)]
pub struct TiledPhysicsPlugin<T: TiledPhysicsBackend>(std::marker::PhantomData<T>);

impl<T: TiledPhysicsBackend> Plugin for TiledPhysicsPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_observer(default_physics_settings::<T>);
        app.add_observer(collider_from_tiles_layer::<T>);
        app.add_observer(collider_from_object::<T>);
    }
}

#[allow(clippy::type_complexity)]
fn default_physics_settings<T: TiledPhysicsBackend>(
    trigger: Trigger<TiledMapCreated>,
    mut commands: Commands,
    q_maps: Query<(Option<&Parent>, Option<&TiledPhysicsSettings<T>>), With<TiledMapMarker>>,
    q_worlds: Query<Option<&TiledPhysicsSettings<T>>, With<TiledWorldMarker>>,
) {
    let map_entity = trigger.event().map;
    if let Ok((parent, settings)) = q_maps.get(map_entity) {
        // Map does not have physics settings
        if settings.is_none() {
            if let Some(settings) = parent.and_then(|p| q_worlds.get(p.get()).ok().flatten()) {
                // Use physics settings from the parent world
                commands.entity(map_entity).insert((*settings).clone());
            } else {
                // Use default settings
                commands
                    .entity(map_entity)
                    .insert(TiledPhysicsSettings::<T>::default());
            }
        }
    }
}

fn collider_from_tiles_layer<T: TiledPhysicsBackend>(
    trigger: Trigger<TiledLayerCreated>,
    mut commands: Commands,
    map_asset: Res<Assets<TiledMap>>,
    q_settings: Query<&TiledPhysicsSettings<T>, With<TiledMapMarker>>,
) {
    let layer = trigger.event().layer(&map_asset);
    if layer.as_tile_layer().is_none() {
        return;
    }

    let Ok(settings) = q_settings.get(trigger.event().map) else {
        return;
    };

    if ObjectNameFilter::from(&settings.tiles_layer_filter).contains(&layer.name) {
        collider::spawn_collider::<T>(
            &settings.backend,
            &mut commands,
            &map_asset,
            &trigger.event().map_handle,
            &TiledColliderSource {
                entity: trigger.event().layer,
                ty: TiledColliderSourceType::new_tiles_layer(trigger.event().layer_id),
            },
        );
    }
}

fn collider_from_object<T: TiledPhysicsBackend>(
    trigger: Trigger<TiledObjectCreated>,
    mut commands: Commands,
    map_asset: Res<Assets<TiledMap>>,
    q_settings: Query<&TiledPhysicsSettings<T>, With<TiledMapMarker>>,
) {
    let layer = trigger.event().layer(&map_asset);
    let object = trigger.event().object(&map_asset);
    let Ok(settings) = q_settings.get(trigger.event().map) else {
        return;
    };

    if ObjectNameFilter::from(&settings.objects_layer_filter).contains(&layer.name)
        && ObjectNameFilter::from(&settings.objects_filter).contains(&object.name)
    {
        collider::spawn_collider::<T>(
            &settings.backend,
            &mut commands,
            &map_asset,
            &trigger.event().map_handle,
            &TiledColliderSource {
                entity: trigger.event().object,
                ty: TiledColliderSourceType::new_object(
                    trigger.event().layer_id,
                    trigger.event().object_id,
                ),
            },
        );
    }
}
