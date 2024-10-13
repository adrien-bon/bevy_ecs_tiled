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
use tiled::{Map, ObjectData};

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
        object_data: &ObjectData,
    ) -> Option<(Vec2, Entity)>;
}

/// Controls physics related settings
#[derive(Clone, Component)]
pub struct TiledPhysicsSettings<T: TiledPhysicsBackend + Default> {
    /// Specify which Tiled object layers to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all objects whose containing layer name matches this filter.
    ///
    /// By default, we add colliders for all objects.
    pub objects_layer_filter: ObjectNames,
    /// Specify which tiles collision object to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all tiles collision objects whose name matches this filter.
    ///
    /// By default, we add colliders for all collision objects.
    pub tiles_objects_filter: ObjectNames,
    /// Physics backend to use.
    pub backend: T,
}

impl<T: TiledPhysicsBackend + Default> Default for TiledPhysicsSettings<T> {
    fn default() -> Self {
        Self {
            objects_layer_filter: ObjectNames::All,
            tiles_objects_filter: ObjectNames::All,
            backend: T::default(),
        }
    }
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
    let Ok(settings) = q_settings.get(trigger.event().map) else {
        return;
    };

    if ObjectNameFilter::from(&settings.objects_layer_filter).contains(&layer.name) {
        let map = trigger.event().map(&map_asset);
        let collider_source =
            TiledColliderSource::new_object(trigger.event().layer_id, trigger.event().object_id);
        if let Some(collider_entity) = collider::spawn_collider::<T>(
            &settings.backend,
            &mut commands,
            map,
            &collider_source,
            &collider_source.object(map).unwrap(),
            trigger.event().object,
            Vec2::ZERO,
        ) {
            commands.trigger(TiledColliderCreated {
                colliders_entities_list: vec![collider_entity],
                map_handle: trigger.event().map_handle.clone(),
                collider_source,
                collider_source_entity: trigger.event().object,
            });
        }
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

    let object_filters = &ObjectNameFilter::from(&settings.tiles_objects_filter);
    let map = trigger.event().map(&map_asset);
    let mut colliders_entities_list = Vec::new();
    let collider_source = TiledColliderSource::new_tile(
        trigger.event().layer_id,
        trigger.event().x,
        trigger.event().y,
    );
    if let Some(collision) = &collider_source.tile(map).unwrap().collision {
        for object_data in collision.object_data().iter() {
            if object_filters.contains(&object_data.name) {
                if let Some(collider_entity) = collider::spawn_collider::<T>(
                    &settings.backend,
                    &mut commands,
                    map,
                    &collider_source,
                    object_data,
                    trigger.event().tile,
                    Vec2 {
                        x: object_data.x - map.tile_width as f32 / 2.,
                        y: (map.tile_height as f32 - object_data.y) - map.tile_height as f32 / 2.,
                    },
                ) {
                    colliders_entities_list.push(collider_entity);
                }
            }
        }
        commands.trigger(TiledColliderCreated {
            colliders_entities_list,
            map_handle: trigger.event().map_handle.clone(),
            collider_source,
            collider_source_entity: trigger.event().tile,
        });
    }
}
