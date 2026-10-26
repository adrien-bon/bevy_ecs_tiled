//! This module handles all things related to physics.
//!
//! It is only available when the `physics` feature is enabled.
//!
//! See the [dedicated book section](https://adrien-bon.github.io/bevy_ecs_tiled/guides/physics.html) for more information.

pub mod backend;
pub mod collider;
pub mod geometry;
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
/// - [`TiledPhysicsRapierBackend`]: for the Rapier 2D physics engine
///
#[derive(Default, Copy, Clone, Debug)]
pub struct TiledPhysicsPlugin<T: TiledPhysicsBackend>(std::marker::PhantomData<T>);

/// When added to a Tiled entity, will prevent tree reduction (combining) of colliders.
///
/// See also [TiledPhysicsSettings::simplify_geometry].
#[derive(Component, Reflect, Copy, PartialEq, Clone, Debug)]
pub struct TiledPhysicsDisableGeometrySimplification;

impl<T: TiledPhysicsBackend> Plugin for TiledPhysicsPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<T>();
        app.register_type::<TiledPhysicsDisableGeometrySimplification>();
        app.add_systems(
            PostUpdate,
            (
                mark_tiles_layers_colliders_dirty::<T>,
                update_tiles_layers_colliders::<T>,
                update_objects_colliders::<T>,
            )
                .chain()
                .in_set(TiledPostUpdateSystems::SyncPhysics),
        );
        app.add_observer(on_tile_removed::<T>);
        app.add_observer(on_object_removed);
        app.add_plugins((backend::plugin, collider::plugin, settings::plugin::<T>));
    }
}

/// Marker [`Component`] for a [`TiledLayer::Tiles`] whose colliders must be updated.
#[derive(Component)]
struct TiledTilesLayerCollidersDirty<T: TiledPhysicsBackend>(std::marker::PhantomData<T>);

impl<T: TiledPhysicsBackend> Default for TiledTilesLayerCollidersDirty<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

/// Mark the parent layer of new or modified tiles so its colliders get updated
fn mark_tiles_layers_colliders_dirty<T: TiledPhysicsBackend>(
    mut commands: Commands,
    tiles_query: Query<
        &ChildOf,
        (
            With<TiledTile>,
            Or<(
                Added<TiledTile>,
                Changed<TiledTilesetId>,
                Changed<TiledTileId>,
                Changed<TilePos>,
                Changed<TileFlip>,
            )>,
        ),
    >,
) {
    let layers = tiles_query
        .iter()
        .map(ChildOf::parent)
        .collect::<bevy::platform::collections::HashSet<_>>();
    for layer in layers {
        commands
            .entity(layer)
            .try_insert(TiledTilesLayerCollidersDirty::<T>::default());
    }
}

/// Mark the parent layer of a removed tile so its colliders get updated
fn on_tile_removed<T: TiledPhysicsBackend>(
    removed: On<Remove<TiledTile>>,
    mut commands: Commands,
    tiles_query: Query<&ChildOf>,
) {
    if let Ok(child_of) = tiles_query.get(removed.entity) {
        commands
            .entity(child_of.parent())
            .try_insert(TiledTilesLayerCollidersDirty::<T>::default());
    }
}

/// Despawn the colliders of an object when its [`TiledObject`] component is removed
fn on_object_removed(
    removed: On<Remove<TiledObject>>,
    mut commands: Commands,
    colliders_query: Query<&TiledColliders>,
) {
    despawn_colliders(
        &mut commands,
        removed.entity,
        colliders_query.get(removed.entity).ok(),
    );
}

/// Despawn previous colliders of a Tiled entity, except if the collider is the entity itself
fn despawn_colliders(commands: &mut Commands, entity: Entity, colliders: Option<&TiledColliders>) {
    for collider in colliders.into_iter().flat_map(|c| c.iter()) {
        if collider != entity {
            // Colliders are also children of their source: they may already be despawned
            commands.entity(collider).try_despawn();
        }
    }
}

/// Spawn again the colliders of tiles layers which have been marked as dirty
fn update_tiles_layers_colliders<T: TiledPhysicsBackend>(
    mut commands: Commands,
    layers_query: Query<
        (
            Entity,
            &TiledLayer,
            &TiledLayerId,
            &TiledName,
            &TiledMapReference,
            Option<&Children>,
            Option<&TiledColliders>,
            Has<TiledPhysicsDisableGeometrySimplification>,
        ),
        With<TiledTilesLayerCollidersDirty<T>>,
    >,
    tiles_query: Query<(&TilePos, &TiledTilesetId, &TiledTileId, &TileFlip), With<TiledTile>>,
    maps_query: Query<(&TiledMap, &TiledPhysicsSettings<T>, &TilemapAnchor)>,
    assets: Res<Assets<TiledMapAsset>>,
    mut message_writer: MessageWriter<TiledEvent<ColliderCreated>>,
) {
    for (
        layer_entity,
        layer,
        layer_id,
        layer_name,
        map_ref,
        children,
        colliders,
        disable_geometry_simplification,
    ) in layers_query.iter()
    {
        commands
            .entity(layer_entity)
            .remove::<TiledTilesLayerCollidersDirty<T>>();

        if !matches!(layer, TiledLayer::Tiles) {
            continue;
        }

        let Ok((tiled_map, settings, anchor)) = maps_query.get(map_ref.0) else {
            warn!(
                "Cannot spawn colliders for entity {layer_entity}: parent map is missing or has no TiledPhysicsSettings"
            );
            continue;
        };
        let Some(map_asset) = assets.get(&tiled_map.0) else {
            continue;
        };

        despawn_colliders(&mut commands, layer_entity, colliders);

        if !settings.tiles_layer_filter.matches(&layer_name.0) {
            continue;
        }

        let tiles = children
            .into_iter()
            .flat_map(|children| children.iter())
            .filter_map(|e| tiles_query.get(e).ok())
            .map(|(pos, tileset_id, tile_id, flip)| (*pos, tileset_id.0, tile_id.0, *flip));
        let polygons =
            collider::tiles_polygons(map_asset, anchor, tiles, &settings.tiles_objects_filter);

        let event = TiledEvent::new(
            layer_entity,
            ColliderCreated::new(TiledColliderSource::TilesLayer, layer_entity),
        )
        .with_map(map_ref.0, tiled_map.0.id())
        .with_layer(layer_entity, layer_id.0)
        .to_owned();
        collider::spawn_colliders::<T>(
            settings,
            &mut commands,
            polygons,
            event,
            &mut message_writer,
            disable_geometry_simplification,
        );
    }
}

/// Spawn again the colliders of new or modified objects
fn update_objects_colliders<T: TiledPhysicsBackend>(
    mut commands: Commands,
    objects_query: Query<
        (
            Entity,
            &TiledObject,
            &TiledObjectId,
            &TiledName,
            &TiledMapReference,
            Option<&ChildOf>,
            Option<&TiledColliders>,
            Has<TiledPhysicsDisableGeometrySimplification>,
        ),
        Changed<TiledObject>,
    >,
    layers_query: Query<(&TiledLayerId, &TiledName), With<TiledLayer>>,
    maps_query: Query<(&TiledMap, &TiledPhysicsSettings<T>)>,
    assets: Res<Assets<TiledMapAsset>>,
    mut message_writer: MessageWriter<TiledEvent<ColliderCreated>>,
) {
    for (
        object_entity,
        object,
        object_id,
        object_name,
        map_ref,
        child_of,
        colliders,
        disable_geometry_simplification,
    ) in objects_query.iter()
    {
        let Ok((tiled_map, settings)) = maps_query.get(map_ref.0) else {
            warn!(
                "Cannot spawn colliders for entity {object_entity}: parent map is missing or has no TiledPhysicsSettings"
            );
            continue;
        };
        let Some(map_asset) = assets.get(&tiled_map.0) else {
            continue;
        };
        let Some((layer_entity, (layer_id, layer_name))) =
            child_of.and_then(|c| layers_query.get(c.parent()).ok().map(|l| (c.parent(), l)))
        else {
            continue;
        };

        despawn_colliders(&mut commands, object_entity, colliders);

        if !settings.objects_layer_filter.matches(&layer_name.0)
            || !settings.objects_filter.matches(&object_name.0)
        {
            continue;
        }

        let polygons = collider::object_polygons(
            map_asset,
            object,
            match object {
                TiledObject::Tile { .. } => &settings.tiles_objects_filter,
                _ => &TiledFilter::All,
            },
        );

        let event = TiledEvent::new(
            object_entity,
            ColliderCreated::new(TiledColliderSource::Object, object_entity),
        )
        .with_map(map_ref.0, tiled_map.0.id())
        .with_layer(layer_entity, layer_id.0)
        .with_object(object_entity, object_id.0)
        .to_owned();
        collider::spawn_colliders::<T>(
            settings,
            &mut commands,
            polygons,
            event,
            &mut message_writer,
            disable_geometry_simplification,
        );
    }
}
