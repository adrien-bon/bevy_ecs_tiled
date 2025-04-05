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

use std::fmt;

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
    Default
    + Clone
    + fmt::Debug
    + 'static
    + std::marker::Sync
    + std::marker::Send
    + FromReflect
    + Reflectable
{
    /// Function responsible for spawning physics colliders
    ///
    /// This function should spawn one or several [Entity] representing a physics
    /// collider and return informations about it.
    /// In case the provided [TiledCollider] is not supported, it should
    /// not spawn anything and return an empty [Vec].
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        tiled_map: &TiledMap,
        filter: &TiledNameFilter,
        collider: &TiledCollider,
        anchor: &TilemapAnchor,
    ) -> Vec<TiledColliderSpawnInfos>;
}

/// Physics related settings.
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
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
#[derive(Default, Copy, Clone, Debug)]
pub struct TiledPhysicsPlugin<T: TiledPhysicsBackend>(std::marker::PhantomData<T>);

impl<T: TiledPhysicsBackend> Plugin for TiledPhysicsPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<TiledColliderMarker>()
            .register_type::<T>()
            .register_type::<TiledPhysicsSettings<T>>()
            .add_systems(
                PreUpdate,
                (
                    initialize_settings_for_worlds::<T>,
                    initialize_settings_for_maps::<T>,
                    collider_from_tiles_layer::<T>,
                    collider_from_object::<T>,
                )
                    .chain()
                    .after(crate::map::process_loaded_maps),
            )
            .add_systems(PostUpdate, update_settings::<T>);
    }
}

fn initialize_settings_for_worlds<T: TiledPhysicsBackend>(
    mut commands: Commands,
    worlds_query: Query<Entity, (With<TiledWorldMarker>, Without<TiledPhysicsSettings<T>>)>,
) {
    for world in worlds_query.iter() {
        commands
            .entity(world)
            .insert(TiledPhysicsSettings::<T>::default());
    }
}

#[allow(clippy::type_complexity)]
fn initialize_settings_for_maps<T: TiledPhysicsBackend>(
    mut commands: Commands,
    maps_query: Query<
        (Entity, Option<&ChildOf>),
        (With<TiledMapMarker>, Without<TiledPhysicsSettings<T>>),
    >,
    worlds_query: Query<&TiledPhysicsSettings<T>, With<TiledWorldMarker>>,
) {
    for (map, parent) in maps_query.iter() {
        commands.entity(map).insert(
            parent
                .and_then(|world| worlds_query.get(world.get()).ok())
                .cloned()
                .unwrap_or_default(),
        );
    }
}

fn update_settings<T: TiledPhysicsBackend>(
    mut commands: Commands,
    maps_query: Query<(Entity, Ref<TiledPhysicsSettings<T>>), With<TiledMapMarker>>,
    worlds_query: Query<(Entity, Ref<TiledPhysicsSettings<T>>), With<TiledWorldMarker>>,
) {
    for (world, settings) in worlds_query.iter() {
        if settings.is_changed() && !settings.is_added() {
            commands.entity(world).insert(RespawnTiledWorld);
        }
    }

    for (map, settings) in maps_query.iter() {
        if settings.is_changed() && !settings.is_added() {
            commands.entity(map).insert(RespawnTiledMap);
        }
    }
}

#[allow(clippy::type_complexity)]
fn collider_from_tiles_layer<T: TiledPhysicsBackend>(
    mut layer_event: EventReader<TiledLayerCreated>,
    mut commands: Commands,
    map_asset: Res<Assets<TiledMap>>,
    maps_query: Query<(&TiledPhysicsSettings<T>, &TilemapAnchor), With<TiledMapMarker>>,
) {
    for ev in layer_event.read() {
        debug!(
            "map entity = {:?}, layer entity = {:?}",
            ev.map.entity, ev.entity
        );
        let (settings, anchor) = maps_query
            .get(ev.map.entity)
            .expect("TiledPhysicsSettings<T> component should be on map entity");
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
                anchor,
            );
        }
    }
}

#[allow(clippy::type_complexity)]
fn collider_from_object<T: TiledPhysicsBackend>(
    mut object_event: EventReader<TiledObjectCreated>,
    mut commands: Commands,
    map_asset: Res<Assets<TiledMap>>,
    maps_query: Query<(&TiledPhysicsSettings<T>, &TilemapAnchor), With<TiledMapMarker>>,
) {
    for ev in object_event.read() {
        let (settings, anchor) = maps_query
            .get(ev.layer.map.entity)
            .expect("TiledPhysicsSettings<T> component should be on map entity");
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
                anchor,
            );
        }
    }
}
