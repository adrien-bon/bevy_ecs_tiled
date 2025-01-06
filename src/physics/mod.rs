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
    ) -> Option<TiledColliderSpawnInfos>;
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
    /// Specify which tiles collision object to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all tiles collision objects whose name matches this filter.
    /// By default, we add colliders for all collision objects.
    pub tiles_objects_filter: ObjectNames,
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
        app.add_observer(collider_from_object::<T>);
        app.add_observer(collider_from_tile::<T>);
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
            Vec2::ZERO,
        );
    }
}

fn collider_from_tile<T: TiledPhysicsBackend>(
    trigger: Trigger<TiledSpecialTileCreated>,
    mut commands: Commands,
    map_asset: Res<Assets<TiledMap>>,
    q_settings: Query<&TiledPhysicsSettings<T>, With<TiledMapMarker>>,
) {
    if let Some(tile_data) = trigger.event().tile(&map_asset).get_tile() {
        if tile_data.collision.is_none() {
            return;
        }
    };

    let Ok(settings) = q_settings.get(trigger.event().map) else {
        return;
    };

    let map = trigger.event().map(&map_asset);
    let layer = trigger.event().layer(&map_asset);
    if settings.tiles_objects_filter == ObjectNames::None
        || !ObjectNameFilter::from(&settings.tiles_layer_filter).contains(&layer.name)
    {
        return;
    }

    let objects_filter = &ObjectNameFilter::from(&settings.tiles_objects_filter);

    if let Some(collision) = trigger
        .event()
        .layer(&map_asset)
        .as_tile_layer()
        .and_then(|tile_layer| {
            tile_layer.get_tile(trigger.event().tiled_index.x, trigger.event().tiled_index.y)
        })
        .and_then(|layer_tile| layer_tile.get_tile())
        .as_ref()
        .and_then(|tile| tile.collision.as_ref())
    {
        // We need to add a Transform to our tile so Transform from
        // the map and layers will be propagated down to the collider(s)
        let world_position = trigger.event().world_position(&map_asset);
        commands
            .entity(trigger.event().tile)
            .insert(Transform::from_xyz(world_position.x, world_position.y, 0.0));

        for (object_id, object_data) in collision.object_data().iter().enumerate() {
            if objects_filter.contains(&object_data.name) {
                collider::spawn_collider::<T>(
                    &settings.backend,
                    &mut commands,
                    &map_asset,
                    &trigger.event().map_handle,
                    &TiledColliderSource {
                        entity: trigger.event().tile,
                        ty: TiledColliderSourceType::new_tile(
                            trigger.event().layer_id,
                            trigger.event().tiled_index.x,
                            trigger.event().tiled_index.y,
                            object_id,
                        ),
                    },
                    Vec2 {
                        x: object_data.x - map.tile_width as f32 / 2.,
                        y: (map.tile_height as f32 - object_data.y) - map.tile_height as f32 / 2.,
                    },
                );
            }
        }
    }
}
