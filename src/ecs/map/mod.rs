//! Tiled maps: spawn a [`TiledMap`] to load and display a `.tmx` map.
//!
//! Settings of the map ([`TiledMapLayerZOffset`], [`TiledMapImageRepeatMargin`], `TilemapAnchor`,
//! `TilemapRenderSettings`) are components of the map entity, and [`TiledMapQuery`] gives access to its
//! layers, tiles and objects entities.
//!
//! See the [spawn / despawn / reload guide](https://adrien-bon.github.io/bevy_ecs_tiled/guides/spawn_reload.html).

pub mod query;
pub(crate) mod spawn;

use crate::{asset::cache::TiledResourceCache, ecs::event::TiledMessageWriters, prelude::*};
use bevy::{asset::RecursiveDependencyLoadState, ecs::system::SystemParam, prelude::*};

/// Main component for loading and managing a Tiled map in the ECS world.
///
/// Attach this component to an entity to load a Tiled map from a `.tmx` file. The inner value is a [`Handle<TiledMapAsset>`],
/// which references the loaded [`TiledMapAsset`]. This entity acts as the root for all layers, tiles, and objects spawned from the map.
///
/// Required components (automatically added with default value if missing), which can be inserted
/// along with the [`TiledMap`] to customize how the map is spawned:
/// - [`TiledMapLayerZOffset`]: Controls Z stacking order between layers.
/// - [`TiledMapImageRepeatMargin`]: Controls image tiling margin for repeated images.
/// - [`TilemapRenderSettings`]: Controls custom parameters for the render pipeline.
/// - [`TilemapAnchor`]: Controls the anchor point of the map.
/// - [`Visibility`] and [`Transform`]: Standard Bevy components.
///
/// The same components are required by a `TiledWorld`, which passes them on to its maps.
///
/// Note that modifying the [`TiledMap`], [`TiledMapLayerZOffset`], [`TilemapRenderSettings`] or [`TilemapAnchor`]
/// components of a map entity respawns the whole map from its asset: all the modifications made to its layers,
/// tiles or objects since it has been spawned are lost.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
///     commands.spawn(TiledMap(asset_server.load("map.tmx")));
/// }
/// ```
#[derive(Component, Reflect, Clone, Debug, Deref, FromTemplate)]
#[reflect(Component, Debug)]
#[require(TiledMapSettings)]
pub struct TiledMap(pub Handle<TiledMapAsset>);

/// Marker [`Component`] requiring the settings components shared by [`TiledMap`] and `TiledWorld` entities.
///
/// Keep the list of these components in the documentation of [`TiledMap`] up to date.
#[derive(Component, Default)]
#[require(
    TiledMapLayerZOffset,
    TiledMapImageRepeatMargin,
    TilemapRenderSettings,
    TilemapAnchor,
    Visibility,
    Transform
)]
pub(crate) struct TiledMapSettings;

/// Controls the Z stacking order between Tiled map layers for correct rendering.
///
/// Attach this component to a [`TiledMap`] entity to specify the Z offset (distance) between each Tiled layer.
/// This ensures that layers are rendered in the correct order and helps prevent Z-fighting artifacts, especially
/// in isometric or multi-layered maps. The value is in world units (typically pixels for 2D maps).
///
/// Defaults to `100.0` units between layers, which is suitable for most 2D maps.
/// - Increase the value if you observe rendering artifacts or Z-fighting between layers.
/// - Decrease the value if you want a more compact stacking of layers (e.g., for a "flatter" map).
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
///     commands.spawn((
///         TiledMap(asset_server.load("map.tmx")),
///         TiledMapLayerZOffset(200.0), // Custom Z offset between layers
///     ));
/// }
/// ```
///
/// # Notes
/// - The Z offset is applied incrementally for each layer: the first layer is at Z=0, the next at Z=offset, etc.
#[derive(Component, Reflect, Copy, Clone, Debug, Deref)]
#[reflect(Component, Default, Debug)]
pub struct TiledMapLayerZOffset(pub f32);

impl Default for TiledMapLayerZOffset {
    fn default() -> Self {
        Self(100.)
    }
}

/// Number of extra tiles to repeat beyond the visible area on each side when tiling an
/// image with repeat x and/or repeat y enabled.
///
/// This margin ensures that, even if the camera moves or the visible area changes slightly,
/// there will be no visible gaps at the edges of the repeated image. The value represents
/// how many additional tiles (in both directions) are rendered outside the current viewport.
/// Increase this value if you observe gaps when moving the camera quickly or zooming out.
#[derive(Component, Reflect, Copy, Clone, Debug, Deref)]
#[reflect(Component, Default, Debug)]
pub struct TiledMapImageRepeatMargin(pub u32);

impl Default for TiledMapImageRepeatMargin {
    fn default() -> Self {
        Self(1)
    }
}

/// Component that stores a reference to the parent Tiled map entity for a given Tiled item.
///
/// This component is automatically attached to all entities that are part of a Tiled map hierarchy,
/// such as layers [`TiledLayer`], tilemaps [`TiledTilemap`], tiles [`TiledTile`], objects [`TiledObject`],
/// and images [`TiledImage`].
/// It allows systems and queries to easily retrieve the root map entity associated with any Tiled sub-entity.
///
/// When spawning a [`TiledTile`] or a [`TiledObject`] yourself, you do not need to insert it: it is
/// automatically inserted from the parent [`TiledLayer`] of the tile or object during [`TiledPostUpdateSystems::First`].
#[derive(Component, Reflect, Copy, Clone, Debug, Deref)]
#[reflect(Component, Debug)]
pub struct TiledMapReference(pub Entity);

/// Marker component to trigger a respawn (reload) of a Tiled map.
///
/// Add this component to the entity holding the [`TiledMap`] to force the map and all its layers, tiles, and objects to be reloaded.
/// This is useful for hot-reloading, resetting, or programmatically refreshing the map state at runtime.
///
/// When present, the plugin will despawn all child entities and re-instantiate the map from its asset, preserving the top-level entity and its components.
/// All the modifications made to the map layers, tiles or objects since it has been spawned are lost.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn respawn_map(mut commands: Commands, map_query: Query<Entity, With<TiledMap>>) {
///     if let Ok(entity) = map_query.single() {
///         commands.entity(entity).insert(RespawnTiledMap);
///     }
/// }
/// ```
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct RespawnTiledMap;

/// [`SystemParam`] giving access to the map of any Tiled entity (map, layer, tile, object, ...).
///
/// It saves you from querying the [`TiledMap`] referenced by the [`TiledMapReference`] of an entity,
/// then retrieving the corresponding [`TiledMapAsset`].
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn log_tiles_positions(tiles: Query<Entity, Added<TiledTile>>, maps: TiledMapHelper) {
///     for tile in tiles.iter() {
///         // Position of the tile center, relative to its tilemap
///         if let Some(position) = maps.tile_relative_position(tile) {
///             info!("Tile {tile} is at {position}");
///         }
///     }
/// }
/// ```
#[derive(SystemParam)]
pub struct TiledMapHelper<'w, 's> {
    maps: Query<'w, 's, (&'static TiledMap, &'static TilemapAnchor)>,
    references: Query<'w, 's, &'static TiledMapReference>,
    tiles: Query<'w, 's, (&'static TilePos, &'static TiledTile)>,
    assets: Res<'w, Assets<TiledMapAsset>>,
}

impl TiledMapHelper<'_, '_> {
    /// Returns the map entity of the given Tiled entity, which can be the map entity itself.
    ///
    /// Returns [`None`] if the entity is neither a map nor has a [`TiledMapReference`].
    pub fn map_entity(&self, entity: Entity) -> Option<Entity> {
        if self.maps.contains(entity) {
            return Some(entity);
        }
        self.references.get(entity).ok().map(|map_ref| map_ref.0)
    }

    /// Returns the [`TiledMapAsset`] and [`TilemapAnchor`] of the map of the given Tiled entity.
    ///
    /// Returns [`None`] if the entity does not belong to a map, or if the map asset is not loaded.
    pub fn map(&self, entity: Entity) -> Option<(&TiledMapAsset, &TilemapAnchor)> {
        let (tiled_map, anchor) = self.maps.get(self.map_entity(entity)?).ok()?;
        Some((self.assets.get(&tiled_map.0)?, anchor))
    }

    /// Returns the position of the center of the given [`TiledTile`] entity, relative to its tilemap.
    ///
    /// See [`TiledMapAsset::tile_relative_position`].
    pub fn tile_relative_position(&self, tile: Entity) -> Option<Vec2> {
        let (tile_pos, tiled_tile) = self.tiles.get(tile).ok()?;
        let (map_asset, anchor) = self.map(tile)?;
        map_asset.tile_relative_position(tile_pos, tiled_tile, anchor)
    }
}

impl std::fmt::Debug for TiledMapHelper<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TiledMapHelper").finish_non_exhaustive()
    }
}

pub(crate) fn plugin(app: &mut bevy::prelude::App) {
    app.register_type::<TiledMap>();
    app.register_type::<TiledMapLayerZOffset>();
    app.register_type::<TiledMapImageRepeatMargin>();
    app.register_type::<TiledMapReference>();
    app.register_type::<RespawnTiledMap>();

    app.add_systems(
        PreUpdate,
        process_loaded_maps.in_set(TiledPreUpdateSystems::ProcessLoadedMaps),
    );
    app.add_systems(
        PostUpdate,
        handle_map_events.in_set(TiledPostUpdateSystems::HandleMapAssetEvents),
    );

    app.add_systems(
        PostUpdate,
        insert_missing_map_references.in_set(TiledPostUpdateSystems::First),
    );
}

/// Retrieve the map of a tile or an object: either its own [`TiledMapReference`] or the one of its parent layer.
pub(crate) fn resolve_map_reference(
    map_ref: Option<&TiledMapReference>,
    child_of: Option<&ChildOf>,
    layers_query: &Query<&TiledMapReference, With<TiledLayer>>,
) -> Option<TiledMapReference> {
    map_ref
        .copied()
        .or_else(|| child_of.and_then(|c| layers_query.get(c.parent()).ok().copied()))
}

/// Insert a [`TiledMapReference`] on new tiles and objects which do not have one,
/// using the one of their parent layer.
fn insert_missing_map_references(
    mut commands: Commands,
    tiles_query: Query<(Entity, Option<&ChildOf>), (Added<TiledTile>, Without<TiledMapReference>)>,
    objects_query: Query<
        (Entity, Option<&ChildOf>),
        (Added<TiledObject>, Without<TiledMapReference>),
    >,
    layers_query: Query<&TiledMapReference, With<TiledLayer>>,
) {
    // No need to warn for tiles: tiles which are not children of a tiles layer are reported when syncing them
    for (entity, child_of) in tiles_query.iter() {
        if let Some(map_ref) = resolve_map_reference(None, child_of, &layers_query) {
            commands.entity(entity).insert(map_ref);
        }
    }
    for (entity, child_of) in objects_query.iter() {
        match resolve_map_reference(None, child_of, &layers_query) {
            Some(map_ref) => {
                commands.entity(entity).insert(map_ref);
            }
            None => warn!(
                "Object {entity} is not a child of an objects layer: it will not have any visual or collider"
            ),
        }
    }
}

/// System to spawn a map once it has been fully loaded.
fn process_loaded_maps(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    maps: Res<Assets<TiledMapAsset>>,
    mut map_query: Query<
        (
            Entity,
            &TiledMap,
            &TilemapRenderSettings,
            &TilemapAnchor,
            &TiledMapLayerZOffset,
        ),
        Or<(
            Changed<TiledMap>,
            Changed<TilemapAnchor>,
            Changed<TiledMapLayerZOffset>,
            Changed<TilemapRenderSettings>,
            With<RespawnTiledMap>,
        )>,
    >,
    layers_query: TiledMapLayers<Entity>,
    mut message_writers: TiledMessageWriters,
) {
    for (map_entity, map_handle, _render_settings, anchor, layer_offset) in map_query.iter_mut() {
        if let Some(load_state) = asset_server.get_recursive_dependency_load_state(&map_handle.0) {
            if !load_state.is_loaded() {
                if let RecursiveDependencyLoadState::Failed(_) = load_state {
                    error!(
                        "Map failed to load, despawn it (handle = {:?})",
                        map_handle.0
                    );
                    commands.entity(map_entity).despawn();
                } else {
                    debug!(
                        "Map is not fully loaded yet, will try again next frame (handle = {:?})",
                        map_handle.0
                    );
                    commands.entity(map_entity).insert(RespawnTiledMap);
                }
                continue;
            }

            // Map should be loaded at this point
            let Some(tiled_map) = maps.get(&map_handle.0) else {
                error!(
                    "Cannot get a valid TiledMapAsset out of Asset<TiledMapAsset>: has the last strong reference to the asset been dropped ? (handle = {:?})",
                    map_handle.0
                );
                commands.entity(map_entity).despawn();
                continue;
            };

            debug!(
                "Map has finished loading, spawn map layers (handle = {:?})",
                map_handle.0
            );

            // Clean previous map layers before trying to spawn the new ones
            for layer in layers_query.iter(map_entity) {
                commands.entity(layer).try_despawn();
            }
            spawn::spawn_map(
                &mut commands,
                map_entity,
                map_handle.0.id(),
                tiled_map,
                layer_offset,
                &mut message_writers,
                anchor,
            );

            // Remove the respawn marker
            commands.entity(map_entity).remove::<RespawnTiledMap>();
        }
    }
}

/// System to update maps as they are changed or removed.
fn handle_map_events(
    mut commands: Commands,
    mut map_events: MessageReader<AssetEvent<TiledMapAsset>>,
    map_query: Query<(Entity, &TiledMap)>,
    mut cache: ResMut<TiledResourceCache>,
) {
    for event in map_events.read() {
        match event {
            AssetEvent::Modified { id } => {
                info!("Map changed: {id}");
                // Note: this call actually clear the cache for the next time we reload an asset
                // That's because the AssetEvent::Modified is sent AFTER the asset is reloaded from disk
                // It means that is the first reload is triggered by a tileset modification, the tileset will
                // not be properly updated since we will still use its previous version in the cache
                cache.clear();
                for (map_entity, map_handle) in map_query.iter() {
                    if map_handle.0.id() == *id {
                        commands.entity(map_entity).insert(RespawnTiledMap);
                    }
                }
            }
            AssetEvent::Removed { id } => {
                info!("Map removed: {id}");
                for (map_entity, map_handle) in map_query.iter() {
                    if map_handle.0.id() == *id {
                        commands.entity(map_entity).despawn();
                    }
                }
            }
            _ => continue,
        }
    }
}
