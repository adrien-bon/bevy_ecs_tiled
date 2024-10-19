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

#[cfg(feature = "user_properties")]
use crate::properties::command::PropertiesCommandExt;

use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::prelude::*;
use tiled::{
    ChunkData, FiniteTileLayer, InfiniteTileLayer, Layer, LayerType, ObjectLayer, Tile, TileId,
    TileLayer,
};

pub(super) fn load_map(
    commands: &mut Commands,
    map_entity: Entity,
    map_handle: &Handle<TiledMap>,
    tiled_map: &TiledMap,
    tiled_id_storage: &mut TiledIdStorage,
    render_settings: &TilemapRenderSettings,
    tiled_settings: &TiledMapSettings,
) {
    commands
        .entity(map_entity)
        .insert(Name::new(format!(
            "TiledMap({} x {})",
            tiled_map.map.width, tiled_map.map.height
        )))
        .insert(TiledMapMarker)
        .insert(SpatialBundle {
            transform: tiled_settings.map_initial_transform,
            visibility: tiled_settings.map_initial_visibility,
            ..SpatialBundle::INHERITED_IDENTITY
        });

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
        offset_z += tiled_settings.layer_z_offset;

        // Apply layer offset and MapPositioning setting
        let offset_transform = Transform::from_xyz(layer.offset_x, -layer.offset_y, offset_z);
        commands.entity(layer_entity).insert(SpatialBundle {
            transform: match &tiled_settings.layer_positioning {
                LayerPositioning::TiledOffset => offset_transform,
                LayerPositioning::Centered => {
                    get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.)
                        * offset_transform
                }
            },
            ..default()
        });

        let layer_infos = TiledLayerCreated {
            map: map_entity,
            layer: layer_entity,
            map_handle: map_handle.clone(),
            layer_id,
        };

        match layer.layer_type() {
            LayerType::Tiles(tile_layer) => {
                commands
                    .entity(layer_entity)
                    .insert(Name::new(format!("TiledMapTileLayer({})", layer.name)))
                    .insert(TiledMapTileLayer);
                load_tiles_layer(
                    commands,
                    tiled_map,
                    &layer_infos,
                    layer,
                    tile_layer,
                    render_settings,
                    &mut tiled_id_storage.tiles,
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
                    tiled_map,
                    &layer_infos,
                    object_layer,
                    &mut tiled_id_storage.objects,
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

        tiled_id_storage.layers.insert(layer.id(), layer_entity);
        layer_events.push(layer_infos);
    }

    #[cfg(feature = "user_properties")]
    {
        let mut props = tiled_map
            .properties
            .clone()
            .hydrate(&tiled_id_storage.objects);

        commands.entity(map_entity).insert_properties(props.map);

        for (id, &entity) in tiled_id_storage.objects.iter() {
            commands
                .entity(entity)
                .insert_properties(props.objects.remove(id).unwrap());
        }

        for (id, &entity) in tiled_id_storage.layers.iter() {
            commands
                .entity(entity)
                .insert_properties(props.layers.remove(id).unwrap());
        }

        for (id, entities) in tiled_id_storage.tiles.iter() {
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
        map: map_entity,
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
    tiled_map: &TiledMap,
    layer_infos: &TiledLayerCreated,
    layer: Layer,
    tile_layer: TileLayer,
    render_settings: &TilemapRenderSettings,
    entity_map: &mut HashMap<(String, TileId), Vec<Entity>>,
    event_list: &mut Vec<TiledSpecialTileCreated>,
) {
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

        let map_type = get_map_type(&tiled_map.map);
        let grid_size = get_grid_size(&tiled_map.map);
        let mut map_size = get_map_size(&tiled_map.map);

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
            .set_parent(layer_infos.layer)
            .id();

        let tile_storage = match tile_layer {
            tiled::TileLayer::Finite(layer_data) => load_finite_tiles_layer(
                commands,
                tiled_map,
                layer_infos,
                layer_for_tileset_entity,
                &layer_data,
                tileset_index,
                tilemap_texture,
                entity_map,
                event_list,
            ),
            tiled::TileLayer::Infinite(layer_data) => {
                let (storage, new_map_size, origin) = load_infinite_tiles_layer(
                    commands,
                    tiled_map,
                    layer_infos,
                    layer_for_tileset_entity,
                    &layer_data,
                    tileset_index,
                    tilemap_texture,
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
                grid_size,
                size: map_size,
                storage: tile_storage,
                texture: tilemap_texture.clone(),
                tile_size,
                spacing: tile_spacing,
                transform: offset_transform,
                map_type,
                render_settings: *render_settings,
                ..Default::default()
            });
    }
}

#[allow(clippy::too_many_arguments)]
fn load_finite_tiles_layer(
    commands: &mut Commands,
    tiled_map: &TiledMap,
    layer_infos: &TiledLayerCreated,
    layer_for_tileset_entity: Entity,
    layer_data: &FiniteTileLayer,
    tileset_index: usize,
    tilemap_texture: &TilemapTexture,
    entity_map: &mut HashMap<(String, TileId), Vec<Entity>>,
    event_list: &mut Vec<TiledSpecialTileCreated>,
) -> TileStorage {
    let grid_size = get_grid_size(&tiled_map.map);
    let map_size = get_map_size(&tiled_map.map);
    let mut tile_storage = TileStorage::empty(map_size);
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
                layer_infos,
                layer_for_tileset_entity,
                tile_entity,
                &tile,
                layer_tile.id(),
                mapped_x,
                mapped_y,
                entity_map,
                event_list,
            );

            tile_storage.set(&tile_pos, tile_entity);
        }
    }
    tile_storage
}

#[allow(clippy::too_many_arguments)]
fn load_infinite_tiles_layer(
    commands: &mut Commands,
    tiled_map: &TiledMap,
    layer_infos: &TiledLayerCreated,
    layer_for_tileset_entity: Entity,
    infinite_layer: &InfiniteTileLayer,
    tileset_index: usize,
    tilemap_texture: &TilemapTexture,
    entity_map: &mut HashMap<(String, TileId), Vec<Entity>>,
    event_list: &mut Vec<TiledSpecialTileCreated>,
) -> (TileStorage, TilemapSize, (f32, f32)) {
    let grid_size = get_grid_size(&tiled_map.map);

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
        "(infinite map) topleft: ({}, {}), bottomright: ({}, {})",
        topleft_x,
        topleft_y,
        bottomright_x,
        bottomright_y
    );

    // TODO: Provide a way to surface the origin point (the point that was 0,0 in Tiled)
    //       to the caller.

    // Recalculate map size based on the top left and bottom right coordinates.
    let map_size = TilemapSize {
        x: (bottomright_x - topleft_x + 1) as u32 * ChunkData::WIDTH,
        y: (bottomright_y - topleft_y + 1) as u32 * ChunkData::HEIGHT,
    };
    log::info!("(infinite map) size: {:?}", map_size);
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
                    y: map_size.y - 1 - tile_y as u32,
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
                    layer_infos,
                    layer_for_tileset_entity,
                    tile_entity,
                    &tile,
                    layer_tile.id(),
                    chunk_pos.0 * ChunkData::WIDTH as i32 + x as i32,
                    chunk_pos.1 * ChunkData::HEIGHT as i32 + y as i32,
                    entity_map,
                    event_list,
                );

                tile_storage.set(&tile_pos, tile_entity);
            }
        }
    }
    (tile_storage, map_size, origin)
}

fn load_objects_layer(
    commands: &mut Commands,
    tiled_map: &TiledMap,
    layer_infos: &TiledLayerCreated,
    object_layer: ObjectLayer,
    entity_map: &mut HashMap<u32, Entity>,
    event_list: &mut Vec<TiledObjectCreated>,
) {
    let map_type = get_map_type(&tiled_map.map);
    let grid_size = get_grid_size(&tiled_map.map);
    let map_size = get_map_size(&tiled_map.map);

    for (object_id, object_data) in object_layer.objects().enumerate() {
        let object_position = from_tiled_coords_to_bevy(
            Vec2::new(object_data.x, object_data.y),
            &map_type,
            &map_size,
            &grid_size,
        );
        let object_entity = commands
            .spawn(SpatialBundle::from_transform(Transform::from_xyz(
                object_position.x,
                object_position.y,
                0.,
            )))
            .insert(Name::new(format!("Object({})", object_data.name)))
            .insert(TiledMapObject)
            .set_parent(layer_infos.layer)
            .id();

        entity_map.insert(object_data.id(), object_entity);
        event_list.push(TiledObjectCreated::from_layer(
            layer_infos,
            object_entity,
            object_id,
        ));
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
    layer_infos: &TiledLayerCreated,
    layer_for_tileset_entity: Entity,
    tile_entity: Entity,
    tile: &Tile,
    tile_id: TileId,
    x: i32,
    y: i32,
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
        is_special_tile = true;
    }

    if is_special_tile {
        event_list.push(TiledSpecialTileCreated::from_layer(
            layer_infos,
            layer_for_tileset_entity,
            tile_entity,
            x,
            y,
        ));
    }
}
