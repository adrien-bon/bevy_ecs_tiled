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

/// `bevy_ecs_tiled` public exports.
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

pub trait TiledPhysicsBackend {
    fn spawn_collider(
        &self,
        commands: &mut Commands,
        map: &Map,
        collider_source: &TiledColliderSource,
    ) -> Option<TiledColliderSpawnInfos>;
}

/// Controls physics related settings
#[derive(Clone, Component, Default)]
pub struct TiledPhysicsSettings<T: TiledPhysicsBackend + Default> {
    /// Specify which Tiled object layers to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all objects whose containing layer name matches this filter.
    ///
    /// By default, we add colliders for all objects.
    pub objects_layer_filter: ObjectNames,
    pub objects_filter: ObjectNames,
    pub tiles_layer_filter: ObjectNames,
    /// Specify which tiles collision object to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all tiles collision objects whose name matches this filter.
    ///
    /// By default, we add colliders for all collision objects.
    pub tiles_objects_filter: ObjectNames,
    /// Physics backend to use.
    pub backend: T,
}

#[derive(Default)]
pub struct TiledPhysicsPlugin<T: TiledPhysicsBackend + Default + std::marker::Sync> {
    backend: std::marker::PhantomData<T>,
}

impl<T: TiledPhysicsBackend + Default + 'static + std::marker::Sync + std::marker::Send> Plugin
    for TiledPhysicsPlugin<T>
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.observe(default_physics_settings::<T>);
        app.observe(collider_from_object::<T>);
        app.observe(collider_from_tile::<T>);
    }
}

fn default_physics_settings<
    T: TiledPhysicsBackend + Default + 'static + std::marker::Sync + std::marker::Send,
>(
    trigger: Trigger<TiledObjectCreated>,
    mut commands: Commands,
    q_settings: Query<&TiledPhysicsSettings<T>, With<TiledMapMarker>>,
) {
    let map_entity = trigger.event().map;
    if q_settings.get(map_entity).is_err() {
        commands
            .entity(map_entity)
            .insert(TiledPhysicsSettings::<T>::default());
    }
}

fn collider_from_object<
    T: TiledPhysicsBackend + Default + 'static + std::marker::Sync + std::marker::Send,
>(
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
            &TiledColliderSource::new_object(trigger.event().layer_id, trigger.event().object_id),
            trigger.event().object,
            Vec2::ZERO,
        );
    }
}

fn collider_from_tile<
    T: TiledPhysicsBackend + Default + 'static + std::marker::Sync + std::marker::Send,
>(
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
        .and_then(|tile_layer| tile_layer.get_tile(trigger.event().x, trigger.event().y))
        .and_then(|layer_tile| layer_tile.get_tile())
        .as_ref()
        .and_then(|tile| tile.collision.as_ref())
    {
        for (object_id, object_data) in collision.object_data().iter().enumerate() {
            if objects_filter.contains(&object_data.name) {
                collider::spawn_collider::<T>(
                    &settings.backend,
                    &mut commands,
                    &map_asset,
                    &trigger.event().map_handle,
                    &TiledColliderSource::new_tile(
                        trigger.event().layer_id,
                        trigger.event().x,
                        trigger.event().y,
                        object_id,
                    ),
                    trigger.event().tile,
                    Vec2 {
                        x: object_data.x - map.tile_width as f32 / 2.,
                        y: (map.tile_height as f32 - object_data.y) - map.tile_height as f32 / 2.,
                    },
                );
            }
        }
    }
}
