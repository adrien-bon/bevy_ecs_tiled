//! This module handles all things related to physics.
//!
//! It is only available when the `physics` feature is enabled.
//!
//! See the [dedicated book section](https://adrien-bon.github.io/bevy_ecs_tiled/guides/physics.html) for more information.

pub mod backend;
pub mod collider;
pub mod geometry;
pub mod settings;

use crate::{ecs::tile::TiledTilePositionVacated, prelude::*};
use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use std::marker::PhantomData;

/// Physics plugin.
///
/// Must be added to your app in order to automatically spawn physics colliders using the provided [`TiledPhysicsBackend`].
///
/// Colliders are derived from tiles and objects entities: they are spawned again during
/// [`TiledPostUpdateSystems::SyncPhysics`] whenever a tile of a layer, or an object, is added, modified or removed.
///
/// Example:
/// ```rust,no_run
/// # #[cfg(feature = "avian")]
/// # {
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// // Using Avian backend for demonstration purpose,
/// // note that we also support TiledPhysicsRapierBackend
/// App::new()
///     .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default());
/// # }
/// ```
///
/// We provide several [`TiledPhysicsBackend`] that can be used out of the box:
/// - [`TiledPhysicsAvianBackend`]: for the Avian 2D physics engine
/// - [`TiledPhysicsRapierBackend`]: for the Rapier 2D physics engine
///
#[derive(Default, Copy, Clone, Debug)]
pub struct TiledPhysicsPlugin<T: TiledPhysicsBackend>(std::marker::PhantomData<T>);

/// When added to a Tiled entity, will prevent merging of its colliders shapes.
///
/// It is taken into account the next time the colliders of this entity are computed: adding or removing
/// it does not spawn colliders again by itself. Insert it when the entity is spawned, for instance in an
/// observer of [`LayerCreated`] or [`ObjectCreated`] events.
///
/// See also [`TiledPhysicsSettings::simplify_geometry`].
#[derive(Component, Reflect, Copy, PartialEq, Clone, Debug)]
pub struct TiledPhysicsDisableGeometrySimplification;

impl<T: TiledPhysicsBackend> Plugin for TiledPhysicsPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<T>();
        app.register_type::<TiledPhysicsDisableGeometrySimplification>();
        // Each tiles layer keeps track of its colliders
        if let Err(error) =
            app.try_register_required_components::<TiledLayer, TiledLayerColliders<T>>()
        {
            warn!("Cannot register tiles layers colliders state: {error}");
        }
        app.add_systems(
            PostUpdate,
            (
                (
                    mark_tiles_colliders_dirty::<T>,
                    mark_colliders_dirty_on_config_change::<T>,
                ),
                update_tiles_layers_colliders::<T>,
                update_objects_colliders::<T>,
            )
                .chain()
                .in_set(TiledPostUpdateSystems::SyncPhysics),
        );
        app.add_observer(on_object_removed::<T>);
        app.add_observer(on_config_removed::<T, TiledColliderStrategy>);
        app.add_observer(on_config_removed::<T, TiledColliderChunkSize>);
        app.add_plugins((backend::plugin, collider::plugin, settings::plugin::<T>));
    }
}

/// Marker [`Component`] for a [`TiledObject`] whose colliders must be updated.
#[derive(Component)]
pub(crate) struct TiledCollidersDirty<T: TiledPhysicsBackend>(PhantomData<T>);

impl<T: TiledPhysicsBackend> Default for TiledCollidersDirty<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Colliders of a [`TiledLayer::Tiles`] for a given physics backend, and what must be updated.
///
/// Colliders are split in chunks of tiles: without chunking, the layer is made of a single chunk.
#[derive(Component)]
pub(crate) struct TiledLayerColliders<T: TiledPhysicsBackend> {
    /// Whether all the colliders of the layer must be updated
    dirty_all: bool,
    /// Positions whose tile has been added, modified or removed since colliders were last updated
    dirty_positions: Vec<TilePos>,
    /// Chunk size used by the current colliders
    chunk_size: Option<UVec2>,
    /// Entities holding the colliders of each chunk
    chunks: HashMap<UVec2, Vec<Entity>>,
    _backend: PhantomData<T>,
}

impl<T: TiledPhysicsBackend> Default for TiledLayerColliders<T> {
    fn default() -> Self {
        Self {
            dirty_all: false,
            dirty_positions: Vec::new(),
            chunk_size: None,
            chunks: HashMap::default(),
            _backend: PhantomData,
        }
    }
}

impl<T: TiledPhysicsBackend> TiledLayerColliders<T> {
    fn is_dirty(&self) -> bool {
        self.dirty_all || !self.dirty_positions.is_empty()
    }

    /// Request an update of all the colliders of the layer
    pub(crate) fn mark_all_dirty(&mut self) {
        self.dirty_all = true;
    }
}

/// Record the positions of new, modified, moved or removed tiles so the colliders of their layer get updated
fn mark_tiles_colliders_dirty<T: TiledPhysicsBackend>(
    tiles_query: Query<
        (&ChildOf, &TilePos),
        (
            With<TiledTile>,
            Or<(Changed<TiledTile>, Changed<TilePos>, Changed<TileFlip>)>,
        ),
    >,
    mut vacated: MessageReader<TiledTilePositionVacated>,
    mut layers_query: Query<&mut TiledLayerColliders<T>>,
) {
    let positions = tiles_query
        .iter()
        .map(|(child_of, position)| (child_of.parent(), *position))
        .chain(
            vacated
                .read()
                .map(|vacated| (vacated.layer, vacated.position)),
        );
    for (layer, position) in positions {
        if let Ok(mut colliders) = layers_query.get_mut(layer) {
            colliders.dirty_positions.push(position);
        }
    }
}

/// Entities whose colliders depend on a [`TiledColliderStrategy`] or [`TiledColliderChunkSize`] component
type CollidersConfigOwner = (
    Entity,
    Option<&'static TiledLayer>,
    Has<TiledObject>,
    Option<&'static Children>,
);

/// Mark the colliders depending on the configuration components of the given layer or object so they get updated
fn mark_config_owner_dirty<T: TiledPhysicsBackend>(
    commands: &mut Commands,
    (entity, layer, is_object, children): (Entity, Option<&TiledLayer>, bool, Option<&Children>),
    objects_query: &Query<(), With<TiledObject>>,
    layers_query: &mut Query<&mut TiledLayerColliders<T>>,
) {
    match layer {
        Some(TiledLayer::Tiles) => {
            if let Ok(mut colliders) = layers_query.get_mut(entity) {
                colliders.dirty_all = true;
            }
        }
        // Objects use the strategy of their layer, unless they have their own
        Some(TiledLayer::Objects) => {
            for object in children
                .into_iter()
                .flat_map(|children| children.iter())
                .filter(|child| objects_query.contains(*child))
            {
                commands
                    .entity(object)
                    .try_insert(TiledCollidersDirty::<T>::default());
            }
        }
        Some(_) => {}
        None if is_object => {
            commands
                .entity(entity)
                .try_insert(TiledCollidersDirty::<T>::default());
        }
        None => {}
    }
}

/// Update the colliders of layers and objects whose [`TiledColliderStrategy`]
/// or [`TiledColliderChunkSize`] has been inserted or modified
fn mark_colliders_dirty_on_config_change<T: TiledPhysicsBackend>(
    mut commands: Commands,
    changed_query: Query<
        CollidersConfigOwner,
        Or<(
            Changed<TiledColliderStrategy>,
            Changed<TiledColliderChunkSize>,
        )>,
    >,
    objects_query: Query<(), With<TiledObject>>,
    mut layers_query: Query<&mut TiledLayerColliders<T>>,
) {
    for owner in changed_query.iter() {
        mark_config_owner_dirty::<T>(&mut commands, owner, &objects_query, &mut layers_query);
    }
}

/// Update the colliders of layers and objects whose configuration component `C`
/// (ie. [`TiledColliderStrategy`] or [`TiledColliderChunkSize`]) has been removed
fn on_config_removed<T: TiledPhysicsBackend, C: Component>(
    removed: On<Remove<C>>,
    mut commands: Commands,
    owners_query: Query<CollidersConfigOwner>,
    objects_query: Query<(), With<TiledObject>>,
    mut layers_query: Query<&mut TiledLayerColliders<T>>,
) {
    if let Ok(owner) = owners_query.get(removed.entity) {
        mark_config_owner_dirty::<T>(&mut commands, owner, &objects_query, &mut layers_query);
    }
}

/// Despawn the colliders of an object when its [`TiledObject`] component is removed
fn on_object_removed<T: TiledPhysicsBackend>(
    removed: On<Remove<TiledObject>>,
    mut commands: Commands,
    colliders_query: Query<&TiledColliders>,
) {
    despawn_colliders::<T>(
        &mut commands,
        removed.entity,
        colliders_query
            .get(removed.entity)
            .into_iter()
            .flat_map(|colliders| colliders.iter()),
    );
}

/// Despawn the given colliders of a source entity (a tiles layer or an object).
///
/// If the collider is the source entity itself, the entity is kept
/// and only its physics and collider components are removed.
fn despawn_colliders<T: TiledPhysicsBackend>(
    commands: &mut Commands,
    source: Entity,
    colliders: impl IntoIterator<Item = Entity>,
) {
    for collider in colliders {
        if collider == source {
            commands.entity(source).try_remove::<(
                T::Collider,
                TiledColliderSource,
                TiledColliderPolygons,
                TiledColliderOf,
            )>();
        } else {
            // Colliders are also descendants of their source: they may already be despawned
            commands.entity(collider).try_despawn();
        }
    }
}

/// Spawn again the colliders of tiles layers, for the chunks whose tiles have been modified
fn update_tiles_layers_colliders<T: TiledPhysicsBackend>(
    mut commands: Commands,
    mut layers_query: Query<
        (
            Entity,
            &TiledLayer,
            &TiledMapReference,
            &TiledTileLayerStorage,
            &mut TiledLayerColliders<T>,
            Option<&TiledColliderStrategy>,
            Option<&TiledColliderChunkSize>,
            Has<TiledPhysicsDisableGeometrySimplification>,
        ),
        With<TiledLayer>,
    >,
    tiles_query: Query<(&TiledTile, &TileFlip)>,
    maps_query: Query<(&TiledMap, &TiledPhysicsSettings<T>, &TilemapAnchor)>,
    assets: Res<Assets<TiledMapAsset>>,
    mut message_writer: MessageWriter<TiledEvent<ColliderCreated>>,
) {
    for (
        layer_entity,
        layer,
        map_ref,
        storage,
        mut colliders,
        strategy,
        chunk_size,
        disable_geometry_simplification,
    ) in layers_query.iter_mut()
    {
        if !colliders.is_dirty() {
            continue;
        }
        if !matches!(layer, TiledLayer::Tiles) {
            colliders.dirty_all = false;
            colliders.dirty_positions.clear();
            continue;
        }

        let Ok((tiled_map, settings, anchor)) = maps_query.get(map_ref.0) else {
            warn!(
                "Cannot spawn colliders for entity {layer_entity}: parent map is missing or has no TiledPhysicsSettings"
            );
            colliders.dirty_all = false;
            colliders.dirty_positions.clear();
            continue;
        };
        // Map asset is not available (yet): keep the layer dirty to try again later
        let Some(map_asset) = assets.get(&tiled_map.0) else {
            continue;
        };

        let dirty_all = std::mem::take(&mut colliders.dirty_all);
        let dirty_positions = std::mem::take(&mut colliders.dirty_positions);

        // Layer own chunk size, then the one of the map
        let chunk_size = chunk_size
            .copied()
            .unwrap_or(settings.tiles_chunk_size)
            .size();
        let chunk_of = |position: &TilePos| {
            chunk_size.map_or(UVec2::ZERO, |size| {
                UVec2::new(position.x, position.y) / size
            })
        };
        // Layer own strategy, then the one of the map
        let strategy = strategy.copied().unwrap_or(settings.collider_strategy);
        let matches = strategy != TiledColliderStrategy::Disabled;

        // Chunks whose colliders must be spawned again
        let mut dirty_chunks = if dirty_all || colliders.chunk_size != chunk_size {
            // Start over
            colliders.chunk_size = chunk_size;
            for (_, entities) in colliders.chunks.drain() {
                despawn_colliders::<T>(&mut commands, layer_entity, entities);
            }
            if matches {
                storage
                    .iter()
                    .map(|(position, _)| chunk_of(&position))
                    .collect::<HashSet<_>>()
            } else {
                HashSet::default()
            }
        } else if matches {
            // The outline of a chunk depends on the tiles around it (see below): a modified tile
            // also makes dirty the adjacent chunks it is next to
            dirty_positions
                .iter()
                .flat_map(|position| {
                    let around =
                        |coord: u32| [coord.saturating_sub(1), coord, coord.saturating_add(1)];
                    around(position.x).into_iter().flat_map(move |x| {
                        around(position.y)
                            .into_iter()
                            .map(move |y| TilePos::new(x, y))
                    })
                })
                .map(|position| chunk_of(&position))
                .collect()
        } else {
            HashSet::default()
        }
        .into_iter()
        .collect::<Vec<_>>();
        // Spawn chunks in a deterministic order
        dirty_chunks.sort_unstable_by_key(|chunk| (chunk.y, chunk.x));
        // Polygons of each kind of tile are the same for all the chunks of the layer
        let mut polygons_cache = collider::TilesPolygonsCache::default();

        for chunk in dirty_chunks {
            if let Some(entities) = colliders.chunks.remove(&chunk) {
                despawn_colliders::<T>(&mut commands, layer_entity, entities);
            }

            // Without chunking, the single chunk covers the whole layer
            let (min, max) = chunk_size.map_or((UVec2::ZERO, UVec2::MAX), |size| {
                let min = chunk.saturating_mul(size);
                (min, min.saturating_add(size - UVec2::ONE))
            });
            let tile_data = |(position, entity): (TilePos, Entity)| {
                tiles_query
                    .get(entity)
                    .ok()
                    .map(|(tile, flip)| (position, tile.tileset_id, tile.tile_id, *flip))
            };
            let tiles = storage
                .iter_in(TilePos::new(min.x, min.y), TilePos::new(max.x, max.y))
                .filter_map(tile_data);
            let polygons = collider::tiles_polygons(
                map_asset,
                anchor,
                tiles,
                &settings.tile_shapes_filter,
                &mut polygons_cache,
            );

            // Geometry of the tiles around the chunk, which belong to adjacent chunks: the edges
            // our colliders share with them are not part of the layer outline.
            // Neighbor tiles are within one tile of the chunk, whatever the map orientation.
            // Note: a tile collision shape which overhangs its own cell by more than a tile
            // is not taken into account.
            let neighbors = if chunk_size.is_some() && !polygons.is_empty() {
                let in_chunk = |position: &TilePos| {
                    (min.x..=max.x).contains(&position.x) && (min.y..=max.y).contains(&position.y)
                };
                let (around_min, around_max) = (
                    min.saturating_sub(UVec2::ONE),
                    max.saturating_add(UVec2::ONE),
                );
                let tiles = storage
                    .iter_in(
                        TilePos::new(around_min.x, around_min.y),
                        TilePos::new(around_max.x, around_max.y),
                    )
                    .filter(|(position, _)| !in_chunk(position))
                    .filter_map(tile_data);
                collider::union_polygons(collider::tiles_polygons(
                    map_asset,
                    anchor,
                    tiles,
                    &settings.tile_shapes_filter,
                    &mut polygons_cache,
                ))
            } else {
                geo::MultiPolygon::new(vec![])
            };

            let event = TiledEvent::new(
                layer_entity,
                ColliderCreated {
                    source: TiledColliderSource::TilesLayer,
                },
            )
            .with_map(map_ref.0, tiled_map.0.id())
            .with_layer(layer_entity);
            let entities = collider::spawn_colliders::<T>(
                settings,
                strategy,
                match chunk_size {
                    Some(_) => collider::ColliderTarget::Chunk(chunk),
                    None => collider::ColliderTarget::Source,
                },
                &mut commands,
                polygons,
                neighbors,
                layer_entity,
                event,
                &mut message_writer,
                disable_geometry_simplification,
            );
            if !entities.is_empty() {
                colliders.chunks.insert(chunk, entities);
            }
        }
    }
}

/// Spawn again the colliders of new or modified objects, and of objects which have been marked as dirty
fn update_objects_colliders<T: TiledPhysicsBackend>(
    mut commands: Commands,
    objects_query: Query<
        (
            Entity,
            &TiledObject,
            &TiledMapReference,
            Option<&ChildOf>,
            Option<&TiledColliders>,
            Option<&TiledColliderStrategy>,
            Has<TiledPhysicsDisableGeometrySimplification>,
            Has<TiledCollidersDirty<T>>,
        ),
        Or<(Changed<TiledObject>, With<TiledCollidersDirty<T>>)>,
    >,
    layers_query: Query<Option<&TiledColliderStrategy>, With<TiledLayer>>,
    maps_query: Query<(&TiledMap, &TiledPhysicsSettings<T>)>,
    assets: Res<Assets<TiledMapAsset>>,
    mut message_writer: MessageWriter<TiledEvent<ColliderCreated>>,
) {
    for (
        object_entity,
        object,
        map_ref,
        child_of,
        colliders,
        object_strategy,
        disable_geometry_simplification,
        dirty,
    ) in objects_query.iter()
    {
        let Ok((tiled_map, settings)) = maps_query.get(map_ref.0) else {
            warn!(
                "Cannot spawn colliders for entity {object_entity}: parent map is missing or has no TiledPhysicsSettings"
            );
            if dirty {
                commands
                    .entity(object_entity)
                    .remove::<TiledCollidersDirty<T>>();
            }
            continue;
        };
        // Map asset is not available (yet): mark the object to try again later
        let Some(map_asset) = assets.get(&tiled_map.0) else {
            if !dirty {
                commands
                    .entity(object_entity)
                    .insert(TiledCollidersDirty::<T>::default());
            }
            continue;
        };
        if dirty {
            commands
                .entity(object_entity)
                .remove::<TiledCollidersDirty<T>>();
        }
        let Some((layer_entity, layer_strategy)) =
            child_of.and_then(|c| layers_query.get(c.parent()).ok().map(|l| (c.parent(), l)))
        else {
            continue;
        };

        despawn_colliders::<T>(
            &mut commands,
            object_entity,
            colliders.into_iter().flat_map(|colliders| colliders.iter()),
        );

        // Object own strategy, then the one of its layer, then the one of the map
        let strategy = object_strategy
            .or(layer_strategy)
            .copied()
            .unwrap_or(settings.collider_strategy);
        if strategy == TiledColliderStrategy::Disabled {
            continue;
        }

        let polygons = collider::object_polygons(
            map_asset,
            object,
            match object {
                TiledObject::Tile { .. } => &settings.tile_shapes_filter,
                _ => &TiledFilter::All,
            },
        );

        let event = TiledEvent::new(
            object_entity,
            ColliderCreated {
                source: TiledColliderSource::Object,
            },
        )
        .with_map(map_ref.0, tiled_map.0.id())
        .with_layer(layer_entity)
        .with_object(object_entity);
        collider::spawn_colliders::<T>(
            settings,
            strategy,
            collider::ColliderTarget::Source,
            &mut commands,
            polygons,
            geo::MultiPolygon::new(vec![]),
            object_entity,
            event,
            &mut message_writer,
            disable_geometry_simplification,
        );
    }
}
