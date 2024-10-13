// The code in this file was originally copied from
// [bevy_ecs_tilemap](https://github.com/StarArawn/bevy_ecs_tilemap).
// The original code is licensed under the following license,
// with modifications under the license in the root of this repository.
//
// --
// MIT License

// Copyright (c) 2021 John

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! This module handles the actual Tiled map loading.

use std::io::{Cursor, Error as IoError, ErrorKind, Read};
#[cfg(feature = "user_properties")]
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "user_properties")]
use crate::properties::{command::PropertiesCommandExt, load::DeserializedMapProperties};

use bevy::{
    asset::{
        io::Reader, AssetLoader, AssetPath, AsyncReadExt, LoadContext, RecursiveDependencyLoadState,
    },
    prelude::*,
    utils::HashMap,
};

#[cfg(feature = "user_properties")]
use bevy::reflect::TypeRegistryArc;

use crate::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use tiled::{
    ChunkData, FiniteTileLayer, InfiniteTileLayer, Layer, LayerType, ObjectLayer, Tile, TileId,
    TileLayer,
};

/// `bevy_ecs_tiled` main `Plugin`.
///
/// This `Plugin` should be added to your application to actually be able to load a Tiled map.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// App::new()
///     .add_plugins(TiledMapPlugin);
/// ```
#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TiledMap>()
            .init_asset_loader::<TiledLoader>()
            .add_systems(Update, (handle_map_events, process_loaded_maps));

        #[cfg(feature = "user_properties")]
        app.add_systems(Startup, export::export_types);

        #[cfg(feature = "physics")]
        app.add_event::<CustomColliderCreationEvent>();
    }
}

/// Tiled map `Asset`.
///
/// `Asset` holding Tiled map informations.
#[derive(TypePath, Asset)]
pub struct TiledMap {
    pub map: tiled::Map,

    pub tilemap_textures: HashMap<usize, TilemapTexture>,

    #[cfg(feature = "user_properties")]
    pub properties: DeserializedMapProperties,

    // The offset into the tileset_images for each tile id within each tileset.
    #[cfg(not(feature = "atlas"))]
    pub tile_image_offsets: HashMap<(usize, tiled::TileId), u32>,
}

struct BytesResourceReader<'a, 'b> {
    bytes: Arc<[u8]>,
    context: &'a mut LoadContext<'b>,
}
impl<'a, 'b> BytesResourceReader<'a, 'b> {
    fn new(bytes: &'a [u8], context: &'a mut LoadContext<'b>) -> Self {
        Self {
            bytes: Arc::from(bytes),
            context,
        }
    }
}

impl<'a, 'b> tiled::ResourceReader for BytesResourceReader<'a, 'b> {
    type Resource = Box<dyn Read + 'a>;
    type Error = IoError;

    fn read_from(&mut self, path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        if let Some(extension) = path.extension() {
            if extension == "tsx" {
                let future = self.context.read_asset_bytes(path.to_path_buf());
                let data = futures_lite::future::block_on(future)
                    .map_err(|err| IoError::new(ErrorKind::NotFound, err))?;
                return Ok(Box::new(Cursor::new(data)));
            }
        }
        Ok(Box::new(Cursor::new(self.bytes.clone())))
    }
}

pub struct TiledLoader {
    #[cfg(feature = "user_properties")]
    pub registry: TypeRegistryArc,
}

impl FromWorld for TiledLoader {
    fn from_world(_world: &mut World) -> Self {
        Self {
            #[cfg(feature = "user_properties")]
            registry: _world.resource::<AppTypeRegistry>().0.clone(),
        }
    }
}

/// [TiledMap] loading error.
#[derive(Debug, thiserror::Error)]
pub enum TiledAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for TiledLoader {
    type Asset = TiledMap;
    type Settings = ();
    type Error = TiledAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let map_path = load_context.path().to_path_buf();

        let map = {
            // Allow the loader to also load tileset images.
            let mut loader = tiled::Loader::with_cache_and_reader(
                tiled::DefaultResourceCache::new(),
                BytesResourceReader::new(&bytes, load_context),
            );
            // Load the map and all tiles.
            loader.load_tmx_map(&map_path).map_err(|e| {
                std::io::Error::new(ErrorKind::Other, format!("Could not load TMX map: {e}"))
            })?
        };

        let mut tilemap_textures = HashMap::default();
        #[cfg(not(feature = "atlas"))]
        let mut tile_image_offsets = HashMap::default();

        for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
            let tilemap_texture = match &tileset.image {
                None => {
                    #[cfg(feature = "atlas")]
                    {
                        log::info!("Skipping image collection tileset '{}' which is incompatible with atlas feature", tileset.name);
                        continue;
                    }

                    #[cfg(not(feature = "atlas"))]
                    {
                        let mut tile_images: Vec<Handle<Image>> = Vec::new();
                        for (tile_id, tile) in tileset.tiles() {
                            if let Some(img) = &tile.image {
                                let asset_path = AssetPath::from(img.source.clone());
                                log::debug!("Loading tile image from {asset_path:?} as image ({tileset_index}, {tile_id})");
                                let texture: Handle<Image> = load_context.load(asset_path.clone());
                                tile_image_offsets
                                    .insert((tileset_index, tile_id), tile_images.len() as u32);
                                tile_images.push(texture.clone());
                            }
                        }

                        TilemapTexture::Vector(tile_images)
                    }
                }
                Some(img) => {
                    let asset_path = AssetPath::from(img.source.clone());
                    let texture: Handle<Image> = load_context.load(asset_path.clone());

                    TilemapTexture::Single(texture.clone())
                }
            };

            tilemap_textures.insert(tileset_index, tilemap_texture);
        }

        #[cfg(feature = "user_properties")]
        let properties =
            DeserializedMapProperties::load(&map, self.registry.read().deref(), load_context);

        #[cfg(feature = "user_properties")]
        dbg!(&properties);

        let asset_map = TiledMap {
            map,
            tilemap_textures,
            #[cfg(feature = "user_properties")]
            properties,
            #[cfg(not(feature = "atlas"))]
            tile_image_offsets,
        };

        log::info!("Loaded map: {}", load_context.path().display());
        Ok(asset_map)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

/// System to update maps as they are changed or removed.
#[allow(clippy::too_many_arguments)]
fn handle_map_events(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<(Entity, &Handle<TiledMap>, &mut TiledIdStorage)>,
    layer_query: Query<(Entity, &TiledMapLayer), With<TiledMapLayer>>,
) {
    for event in map_events.read() {
        match event {
            AssetEvent::Modified { id } => {
                log::info!("Map changed: {id}");
                for (map_entity, map_handle, _) in map_query.iter() {
                    if map_handle.id() == *id {
                        commands.entity(map_entity).insert(RespawnTiledMap);
                    }
                }
            }
            AssetEvent::Removed { id } => {
                log::info!("Map removed: {id}");
                remove_map_by_asset_id(
                    &mut commands,
                    &tile_storage_query,
                    &mut map_query,
                    &layer_query,
                    id,
                );
            }
            _ => continue,
        }
    }
}

/// System to spawn a map once it has been fully loaded.
#[allow(clippy::type_complexity)]
fn process_loaded_maps(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    maps: ResMut<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<
        (
            Entity,
            &Handle<TiledMap>,
            &mut TiledIdStorage,
            &TilemapRenderSettings,
            &TiledMapSettings,
        ),
        Or<(Changed<Handle<TiledMap>>, With<RespawnTiledMap>)>,
    >,
) {
    for (map_entity, map_handle, mut layer_storage, render_settings, tiled_settings) in
        map_query.iter_mut()
    {
        if let Some(load_state) = asset_server.get_recursive_dependency_load_state(map_handle) {
            if load_state != RecursiveDependencyLoadState::Loaded {
                // If not fully loaded yet, insert the 'Respawn' marker so we will try to load it at next frame
                commands.entity(map_entity).insert(RespawnTiledMap);
                debug!("Map {:?} is not yet loaded...", map_handle.path());
                continue;
            }
            if let Some(tiled_map) = maps.get(map_handle) {
                info!("Spawning map {:?}", map_handle.path());
                commands.entity(map_entity).insert(SpatialBundle::default());
                remove_layers(&mut commands, &tile_storage_query, &mut layer_storage);
                load_map(
                    &mut commands,
                    &mut layer_storage,
                    map_entity,
                    map_handle,
                    tiled_map,
                    render_settings,
                    tiled_settings,
                );
                commands.entity(map_entity).remove::<RespawnTiledMap>();
            }
        }
    }
}

fn remove_map_by_asset_id(
    commands: &mut Commands,
    tile_storage_query: &Query<(Entity, &TileStorage)>,
    map_query: &mut Query<(Entity, &Handle<TiledMap>, &mut TiledIdStorage)>,
    layer_query: &Query<(Entity, &TiledMapLayer), With<TiledMapLayer>>,
    asset_id: &AssetId<TiledMap>,
) {
    log::info!("removing map by asset id: {}", asset_id);
    for (_, map_handle, mut layer_storage) in map_query.iter_mut() {
        log::info!("checking layer to remove: {}", map_handle.id());

        // Only process the map that was removed.
        if map_handle.id() != *asset_id {
            continue;
        }

        remove_layers(commands, tile_storage_query, &mut layer_storage);
    }

    // Also manually despawn layers for this map.
    // This is necessary because when a new layer is added, the map handle
    // generation is incremented, and then a subsequent removal event will not
    // match the map_handle in the loop above.
    for (layer_entity, map_layer) in layer_query.iter() {
        // only deal with currently changed map
        if map_layer.map_handle_id != *asset_id {
            continue;
        }

        commands.entity(layer_entity).despawn_recursive();
    }
}

fn remove_layers(
    commands: &mut Commands,
    tile_storage_query: &Query<(Entity, &TileStorage)>,
    layer_storage: &mut TiledIdStorage,
) {
    for layer_entity in layer_storage.layers.values() {
        if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
            for tile in layer_tile_storage.iter().flatten() {
                commands.entity(*tile).despawn_recursive()
            }
        }
        commands.entity(*layer_entity).despawn_recursive();
    }
    layer_storage.layers.clear();
}

#[allow(clippy::too_many_arguments, unused_mut)]
fn load_map(
    mut commands: &mut Commands,
    layer_storage: &mut TiledIdStorage,
    map_entity: Entity,
    map_handle: &Handle<TiledMap>,
    tiled_map: &TiledMap,
    render_settings: &TilemapRenderSettings,
    tiled_settings: &TiledMapSettings,
) {
    commands
        .entity(map_entity)
        .insert(Visibility::Visible)
        .insert(InheritedVisibility::VISIBLE)
        .insert(Name::new(format!(
            "TiledMap({} x {})",
            tiled_map.map.width, tiled_map.map.height
        )))
        .insert(TiledMapMarker);

    let map_type = get_map_type(&tiled_map.map);
    let map_size = get_map_size(&tiled_map.map);
    let grid_size = get_grid_size(&tiled_map.map);

    let mut layer_events: Vec<TiledLayerCreated> = Vec::new();
    let mut object_events: Vec<TiledObjectCreated> = Vec::new();
    let mut special_tile_events: Vec<TiledSpecialTileCreated> = Vec::new();

    // Order of the differents layers in the .TMX file is important:
    // a layer appearing last in the .TMX should appear "on top" of previous layers
    // Start with a negative offset so the upper layer will be at Z-offset = 0
    let mut offset_z = tiled_map.map.layers().len() as f32 * (-100.0);

    // Once materials have been created/added we need to then create the layers.
    for (layer_id, layer) in tiled_map.map.layers().enumerate() {
        // Spawn layer entity and attach it to the map entity
        let layer_entity = commands
            .spawn((
                TiledMapLayer {
                    map_handle_id: map_handle.id(),
                },
                TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
            ))
            .set_parent(map_entity)
            .id();

        // Increment Z offset
        offset_z += 100.;

        // Apply layer offset and MapPositioning setting
        let offset_transform = Transform::from_xyz(layer.offset_x, -layer.offset_y, offset_z);
        commands.entity(layer_entity).insert(SpatialBundle {
            transform: match &tiled_settings.map_positioning {
                MapPositioning::LayerOffset => offset_transform,
                MapPositioning::Centered => {
                    get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.)
                        * offset_transform
                }
            },
            ..default()
        });

        match layer.layer_type() {
            LayerType::Tiles(tile_layer) => {
                commands
                    .entity(layer_entity)
                    .insert(Name::new(format!("TiledMapTileLayer({})", layer.name)))
                    .insert(TiledMapTileLayer);
                load_tiles_layer(
                    commands,
                    map_handle,
                    layer_id,
                    layer_entity,
                    layer,
                    tile_layer,
                    tiled_map,
                    &map_type,
                    &map_size,
                    &grid_size,
                    render_settings,
                    tiled_settings,
                    &mut layer_storage.tiles,
                    &mut special_tile_events,
                );
            }
            LayerType::Objects(object_layer) => {
                commands
                    .entity(layer_entity)
                    .insert(Name::new(format!("TiledMapObjectLayer({})", layer.name)))
                    .insert(TiledMapObjectLayer);
                load_objects_layer(
                    commands,
                    map_handle,
                    layer_id,
                    layer_entity,
                    layer,
                    object_layer,
                    &map_type,
                    &map_size,
                    &grid_size,
                    tiled_settings,
                    &mut layer_storage.objects,
                    &mut object_events,
                );
            }
            LayerType::Group(_group_layer) => {
                commands
                    .entity(layer_entity)
                    .insert(Name::new(format!("TiledMapGroupLayer({})", layer.name)))
                    .insert(TiledMapGroupLayer);
                // TODO: not implemented yet.
            }
            LayerType::Image(_image_layer) => {
                commands
                    .entity(layer_entity)
                    .insert(Name::new(format!("TiledMapImageLayer({})", layer.name)))
                    .insert(TiledMapImageLayer);
                // TODO: not implemented yet.
            }
        };

        layer_storage.layers.insert(layer.id(), layer_entity);
        layer_events.push(TiledLayerCreated {
            entity: layer_entity,
            map_handle: map_handle.clone(),
            layer_id,
        })
    }

    #[cfg(feature = "user_properties")]
    {
        let mut props = tiled_map.properties.clone().hydrate(&layer_storage.objects);

        commands.entity(map_entity).insert_properties(props.map);

        for (id, &entity) in layer_storage.objects.iter() {
            commands
                .entity(entity)
                .insert_properties(props.objects.remove(id).unwrap());
        }

        for (id, &entity) in layer_storage.layers.iter() {
            commands
                .entity(entity)
                .insert_properties(props.layers.remove(id).unwrap());
        }

        for (id, entities) in layer_storage.tiles.iter() {
            let Some(p) = props.tiles.get(&id.0).and_then(|e| e.get(&id.1)) else {
                continue;
            };
            for &entity in entities {
                commands.entity(entity).insert_properties(p.clone());
            }
        }
    }

    // Send events
    commands.trigger(TiledMapCreated {
        entity: map_entity,
        map_handle: map_handle.clone(),
    });
    for e in layer_events {
        commands.trigger(e);
    }
    for e in object_events {
        commands.trigger(e);
    }
    for e in special_tile_events {
        commands.trigger(e);
    }
}

#[allow(clippy::too_many_arguments)]
fn load_tiles_layer(
    commands: &mut Commands,
    map_handle: &Handle<TiledMap>,
    layer_id: usize,
    layer_entity: Entity,
    layer: Layer,
    tile_layer: TileLayer,
    tiled_map: &TiledMap,
    map_type: &TilemapType,
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
    render_settings: &TilemapRenderSettings,
    tiled_settings: &TiledMapSettings,
    entity_map: &mut HashMap<(String, TileId), Vec<Entity>>,
    event_list: &mut Vec<TiledSpecialTileCreated>,
) -> TilemapSize {
    // The TilemapBundle requires that all tile images come exclusively from a single
    // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
    // the per-tile images must be the same size. Since Tiled allows tiles of mixed
    // tilesets on each layer and allows differently-sized tile images in each tileset,
    // this means we need to load each combination of tileset and layer separately.
    for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
        let Some(tilemap_texture) = tiled_map.tilemap_textures.get(&tileset_index) else {
            log::warn!("Skipped creating layer with missing tilemap textures.");
            continue;
        };

        let mut map_size = *map_size;

        let tile_size = TilemapTileSize {
            x: tileset.tile_width as f32,
            y: tileset.tile_height as f32,
        };

        let tile_spacing = TilemapSpacing {
            x: tileset.spacing as f32,
            y: tileset.spacing as f32,
        };

        let mut offset_x = 0.;
        let mut offset_y = 0.;

        let layer_for_tileset_entity = commands
            .spawn((
                Name::new(format!(
                    "TiledMapTileLayerForTileset({}, {})",
                    layer.name, tileset.name
                )),
                TiledMapTileLayerForTileset,
            ))
            .set_parent(layer_entity)
            .id();

        let tile_storage = match tile_layer {
            tiled::TileLayer::Finite(layer_data) => load_finite_tiles_layer(
                commands,
                map_handle,
                layer_id,
                layer_for_tileset_entity,
                tiled_map,
                map_type,
                &map_size,
                grid_size,
                &layer_data,
                tileset_index,
                tilemap_texture,
                tiled_settings,
                entity_map,
                event_list,
            ),
            tiled::TileLayer::Infinite(layer_data) => {
                let (storage, new_map_size, origin) = load_infinite_tiles_layer(
                    commands,
                    map_handle,
                    layer_id,
                    layer_for_tileset_entity,
                    tiled_map,
                    map_type,
                    grid_size,
                    &layer_data,
                    tileset_index,
                    tilemap_texture,
                    tiled_settings,
                    entity_map,
                    event_list,
                );
                map_size = new_map_size;
                // log::info!("Infinite layer origin: {:?}", origin);
                offset_x += origin.0 * grid_size.x;
                offset_y -= origin.1 * grid_size.y;
                storage
            }
        };

        let offset_transform = Transform::from_xyz(
            offset_x + grid_size.x / 2.,
            -offset_y + grid_size.y / 2.,
            0.,
        );

        commands
            .entity(layer_for_tileset_entity)
            .insert(TilemapBundle {
                grid_size: *grid_size,
                size: map_size,
                storage: tile_storage,
                texture: tilemap_texture.clone(),
                tile_size,
                spacing: tile_spacing,
                transform: offset_transform,
                map_type: *map_type,
                render_settings: *render_settings,
                ..Default::default()
            });
    }
    *map_size
}

#[allow(clippy::too_many_arguments)]
fn load_finite_tiles_layer(
    commands: &mut Commands,
    map_handle: &Handle<TiledMap>,
    layer_id: usize,
    layer_for_tileset_entity: Entity,
    tiled_map: &TiledMap,
    map_type: &TilemapType,
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
    layer_data: &FiniteTileLayer,
    tileset_index: usize,
    tilemap_texture: &TilemapTexture,
    tiled_settings: &TiledMapSettings,
    entity_map: &mut HashMap<(String, TileId), Vec<Entity>>,
    event_list: &mut Vec<TiledSpecialTileCreated>,
) -> TileStorage {
    let mut tile_storage = TileStorage::empty(*map_size);
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            // Transform TMX coords into bevy coords.
            let mapped_y = tiled_map.map.height - 1 - y;

            let mapped_x = x as i32;
            let mapped_y = mapped_y as i32;

            let Some(layer_tile) = layer_data.get_tile(mapped_x, mapped_y) else {
                continue;
            };
            if tileset_index != layer_tile.tileset_index() {
                continue;
            }
            let Some(layer_tile_data) = layer_data.get_tile_data(mapped_x, mapped_y) else {
                continue;
            };
            let Some(tile) = layer_tile.get_tile() else {
                continue;
            };

            let texture_index = match tilemap_texture {
                TilemapTexture::Single(_) => layer_tile.id(),
                #[cfg(not(feature = "atlas"))]
                TilemapTexture::Vector(_) =>
                    *tiled_map.tile_image_offsets.get(&(tileset_index, layer_tile.id()))
                        .expect("The offset into to image vector should have been saved during the initial load."),
                #[cfg(not(feature = "atlas"))]
                _ => unreachable!()
            };

            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(layer_for_tileset_entity),
                    texture_index: TileTextureIndex(texture_index),
                    flip: TileFlip {
                        x: layer_tile_data.flip_h,
                        y: layer_tile_data.flip_v,
                        d: layer_tile_data.flip_d,
                    },
                    ..Default::default()
                })
                .insert(SpatialBundle::from_transform(Transform::from_xyz(
                    tile_pos.x as f32 * grid_size.x,
                    tile_pos.y as f32 * grid_size.y,
                    0.0,
                )))
                .set_parent(layer_for_tileset_entity)
                .insert(Name::new(format!(
                    "TiledMapTile({},{})",
                    tile_pos.x, tile_pos.y
                )))
                .insert(TiledMapTile)
                .id();

            handle_special_tile(
                commands,
                map_handle,
                layer_id,
                tile_entity,
                &tile,
                layer_tile.id(),
                mapped_x,
                mapped_y,
                tiled_settings,
                map_type,
                grid_size,
                entity_map,
                event_list,
            );

            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    tile_storage
}

#[allow(clippy::too_many_arguments, unused)]
fn load_infinite_tiles_layer(
    commands: &mut Commands,
    map_handle: &Handle<TiledMap>,
    layer_id: usize,
    layer_for_tileset_entity: Entity,
    tiled_map: &TiledMap,
    map_type: &TilemapType,
    grid_size: &TilemapGridSize,
    infinite_layer: &InfiniteTileLayer,
    tileset_index: usize,
    tilemap_texture: &TilemapTexture,
    tiled_settings: &TiledMapSettings,
    entity_map: &mut HashMap<(String, TileId), Vec<Entity>>,
    event_list: &mut Vec<TiledSpecialTileCreated>,
) -> (TileStorage, TilemapSize, (f32, f32)) {
    // Determine top left coordinate so we can offset the map.
    let (topleft_x, topleft_y) = infinite_layer
        .chunks()
        .fold((999999, 999999), |acc, (pos, _)| {
            (acc.0.min(pos.0), acc.1.min(pos.1))
        });

    let (bottomright_x, bottomright_y) = infinite_layer
        .chunks()
        .fold((topleft_x, topleft_y), |acc, (pos, _)| {
            (acc.0.max(pos.0), acc.1.max(pos.1))
        });

    log::info!(
        "topleft: ({}, {}), bottomright: ({}, {})",
        topleft_x,
        topleft_y,
        bottomright_x,
        bottomright_y
    );

    #[cfg(feature = "physics")]
    let collision_object_names =
        crate::prelude::ObjectNameFilter::from(&tiled_settings.collision_object_names);

    // TODO: Provide a way to surface the origin point (the point that was 0,0 in Tiled)
    //       to the caller.

    // Recalculate map size based on the top left and bottom right coordinates.
    let map_size = TilemapSize {
        x: (bottomright_x - topleft_x + 1) as u32 * ChunkData::WIDTH,
        y: (bottomright_y - topleft_y + 1) as u32 * ChunkData::HEIGHT,
    };
    log::info!("map size: {:?}", map_size);
    let origin = (
        topleft_x as f32 * ChunkData::WIDTH as f32,
        ((topleft_y as f32 / 2.) * ChunkData::HEIGHT as f32) + 1.,
    );

    let mut tile_storage = TileStorage::empty(map_size);

    for (chunk_pos, chunk) in infinite_layer.chunks() {
        // bevy_ecs_tilemap doesn't support negative tile coordinates, so shift all chunks
        // such that the top-left chunk is at (0, 0).
        let chunk_pos_mapped = (chunk_pos.0 - topleft_x, chunk_pos.1 - topleft_y);

        for x in 0..ChunkData::WIDTH {
            for y in 0..ChunkData::HEIGHT {
                // Invert y to match bevy coordinates.
                let Some(layer_tile) = chunk.get_tile(x as i32, y as i32) else {
                    continue;
                };
                if tileset_index != layer_tile.tileset_index() {
                    continue;
                }
                let Some(layer_tile_data) = chunk.get_tile_data(x as i32, y as i32) else {
                    continue;
                };
                let Some(tile) = layer_tile.get_tile() else {
                    continue;
                };

                let (tile_x, tile_y) = (
                    chunk_pos_mapped.0 * ChunkData::WIDTH as i32 + x as i32,
                    chunk_pos_mapped.1 * ChunkData::HEIGHT as i32 + y as i32,
                );

                let texture_index = match tilemap_texture {
                    TilemapTexture::Single(_) => layer_tile.id(),
                    #[cfg(not(feature = "atlas"))]
                    TilemapTexture::Vector(_) =>
                        *tiled_map.tile_image_offsets.get(&(tileset_index, layer_tile.id()))
                            .expect("The offset into to image vector should have been saved during the initial load."),
                    #[cfg(not(feature = "atlas"))]
                    _ => unreachable!()
                };

                let tile_pos = TilePos {
                    x: tile_x as u32,
                    y: map_size.y - tile_y as u32,
                };
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(layer_for_tileset_entity),
                        texture_index: TileTextureIndex(texture_index),
                        flip: TileFlip {
                            x: layer_tile_data.flip_h,
                            y: layer_tile_data.flip_v,
                            d: layer_tile_data.flip_d,
                        },
                        ..Default::default()
                    })
                    .insert(SpatialBundle::from_transform(Transform::from_xyz(
                        tile_pos.x as f32 * grid_size.x,
                        tile_pos.y as f32 * grid_size.y,
                        0.0,
                    )))
                    .set_parent(layer_for_tileset_entity)
                    .insert(Name::new(format!("Tile({},{})", tile_pos.x, tile_pos.y)))
                    .insert(TiledMapTile)
                    .id();
                handle_special_tile(
                    commands,
                    map_handle,
                    layer_id,
                    tile_entity,
                    &tile,
                    layer_tile.id(),
                    tile_x,
                    tile_y,
                    tiled_settings,
                    map_type,
                    grid_size,
                    entity_map,
                    event_list,
                );

                tile_storage.set(&tile_pos, tile_entity);
            }
        }
    }

    (tile_storage, map_size, origin)
}

#[allow(clippy::too_many_arguments)]
fn load_objects_layer(
    commands: &mut Commands,
    map_handle: &Handle<TiledMap>,
    layer_id: usize,
    layer_entity: Entity,
    _layer: Layer,
    object_layer: ObjectLayer,
    map_type: &TilemapType,
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
    _tiled_settings: &TiledMapSettings,
    entity_map: &mut HashMap<u32, Entity>,
    event_list: &mut Vec<TiledObjectCreated>,
) {
    #[cfg(feature = "physics")]
    let collision_layer_names =
        crate::prelude::ObjectNameFilter::from(&_tiled_settings.collision_layer_names);

    for (object_id, object_data) in object_layer.objects().enumerate() {
        let object_position = match map_type {
            TilemapType::Hexagon(HexCoordSystem::ColumnOdd) => Vec2::new(
                object_data.x + grid_size.x / 4.,
                (map_size.y as f32 + 0.5) * grid_size.y - object_data.y,
            ),
            TilemapType::Hexagon(HexCoordSystem::ColumnEven) => Vec2::new(
                object_data.x + grid_size.x / 4.,
                (map_size.y as f32 + 0.) * grid_size.y - object_data.y,
            ),
            TilemapType::Hexagon(HexCoordSystem::RowOdd) => Vec2::new(
                object_data.x,
                map_size.y as f32 * grid_size.y * 0.75 + grid_size.y / 4. - object_data.y,
            ),
            TilemapType::Hexagon(HexCoordSystem::RowEven) => Vec2::new(
                object_data.x - grid_size.x / 2.,
                map_size.y as f32 * grid_size.y * 0.75 + grid_size.y / 4. - object_data.y,
            ),
            TilemapType::Isometric(coords_system) => from_isometric_coords_to_bevy(
                *coords_system,
                Vec2::new(object_data.x, object_data.y),
                map_size,
                grid_size,
            ),
            _ => Vec2::new(
                object_data.x,
                map_size.y as f32 * grid_size.y - object_data.y,
            ),
        };

        let object_entity = commands
            .spawn(SpatialBundle::from_transform(Transform::from_xyz(
                object_position.x,
                object_position.y,
                0.,
            )))
            .insert(Name::new(format!("Object({})", object_data.name)))
            .insert(TiledMapObject)
            .set_parent(layer_entity)
            .id();

        entity_map.insert(object_data.id(), object_entity);
        event_list.push(TiledObjectCreated {
            entity: object_entity,
            map_handle: map_handle.clone(),
            layer_id,
            object_id,
        });

        #[cfg(feature = "physics")]
        {
            let physics_backend = &_tiled_settings.physics_backend;
            if collision_layer_names.contains(&_layer.name.trim().to_lowercase()) {
                physics_backend.insert_object_colliders(
                    commands,
                    object_entity,
                    map_type,
                    &object_data,
                    _tiled_settings.collider_callback,
                );
            }
        }
    }
}

fn get_animated_tile(tile: &Tile) -> Option<AnimatedTile> {
    let Some(animation_data) = &tile.animation else {
        return None;
    };
    let mut previous_tile_id = None;
    let first_tile = animation_data.iter().next()?;
    let last_tile = animation_data.iter().last()?;

    // Sanity checks: current limitations from bevy_ecs_tilemap
    for frame in animation_data {
        if frame.duration != first_tile.duration {
            log::warn!("Animated tile with non constant frame duration is currently not supported");
            return None;
        }
        if let Some(id) = previous_tile_id {
            if frame.tile_id != id + 1 {
                log::warn!("Animated tile with non-aligned frame tiles is currently not supported");
                return None;
            }
        }
        previous_tile_id = Some(frame.tile_id);
    }

    Some(AnimatedTile {
        start: first_tile.tile_id,
        end: last_tile.tile_id,
        speed: 1000. / first_tile.duration as f32, // duration is in ms and we want a 'frames per second' speed
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_special_tile(
    commands: &mut Commands,
    map_handle: &Handle<TiledMap>,
    layer_id: usize,
    tile_entity: Entity,
    tile: &Tile,
    tile_id: TileId,
    x: i32,
    y: i32,
    _tiled_settings: &TiledMapSettings,
    _map_type: &TilemapType,
    _grid_size: &TilemapGridSize,
    entity_map: &mut HashMap<(String, TileId), Vec<Entity>>,
    event_list: &mut Vec<TiledSpecialTileCreated>,
) {
    let mut is_special_tile = false;

    // Handle animated tiles
    if let Some(animated_tile) = get_animated_tile(tile) {
        commands.entity(tile_entity).insert(animated_tile);
    }

    // Handle custom tiles (with user properties)
    if !tile.properties.is_empty() {
        let key = (tile.tileset().name.clone(), tile_id);
        entity_map
            .entry(key)
            .and_modify(|entities| {
                entities.push(tile_entity);
            })
            .or_insert(vec![tile_entity]);
        is_special_tile = true;
    }

    // Handle tiles with collision
    if let Some(_collision) = tile.collision.as_ref() {
        #[cfg(feature = "physics")]
        {
            let physics_backend = &_tiled_settings.physics_backend;
            physics_backend.insert_tile_colliders(
                commands,
                &ObjectNameFilter::from(&_tiled_settings.collision_object_names),
                tile_entity,
                _map_type,
                _grid_size,
                _collision,
                _tiled_settings.collider_callback,
            );
        }
        is_special_tile = true;
    }

    if is_special_tile {
        event_list.push(TiledSpecialTileCreated {
            entity: tile_entity,
            map_handle: map_handle.clone(),
            layer_id,
            x,
            y,
        });
    }
}
