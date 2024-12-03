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
use bevy::{prelude::*, sprite::Anchor, utils::HashMap};
use bevy_ecs_tilemap::prelude::*;
use tiled::{
    ChunkData, FiniteTileLayer, ImageLayer, InfiniteTileLayer, Layer, LayerType, ObjectLayer, Tile,
    TileId, TileLayer, TilesetLocation,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn load_map(
    commands: &mut Commands,
    map_entity: Entity,
    map_handle: &Handle<TiledMap>,
    tiled_map: &TiledMap,
    tiled_id_storage: &mut TiledIdStorage,
    render_settings: &TilemapRenderSettings,
    tiled_settings: &TiledMapSettings,
    asset_server: &Res<AssetServer>,
) {
    commands.entity(map_entity).insert((
        Name::new(format!(
            "TiledMap({} x {})",
            tiled_map.map.width, tiled_map.map.height
        )),
        TiledMapMarker,
    ));

    let map_type = get_map_type(&tiled_map.map);
    let map_size = get_map_size(&tiled_map.map);
    let grid_size = get_grid_size(&tiled_map.map);

    let mut layer_events: Vec<TiledLayerCreated> = Vec::new();
    let mut object_events: Vec<TiledObjectCreated> = Vec::new();
    let mut special_tile_events: Vec<TiledSpecialTileCreated> = Vec::new();

    // Order of the differents layers in the .TMX file is important:
    // a layer appearing last in the .TMX should appear "on top" of previous layers
    // Start with a negative offset so the upper layer will be at Z-offset = 0
    let mut offset_z = tiled_map.map.layers().len() as f32 * (-tiled_settings.layer_z_offset);

    // Once materials have been created/added we need to then create the layers.
    for (layer_id, layer) in tiled_map.map.layers().enumerate() {
        // Increment Z offset and compute layer transform offset
        offset_z += tiled_settings.layer_z_offset;
        let offset_transform = Transform::from_xyz(layer.offset_x, -layer.offset_y, offset_z);

        // Spawn layer entity and attach it to the map entity
        let layer_entity = commands
            .spawn((
                TiledMapLayer {
                    map_handle_id: map_handle.id(),
                },
                // Apply layer offset and MapPositioning setting
                match &tiled_settings.layer_positioning {
                    LayerPositioning::TiledOffset => offset_transform,
                    LayerPositioning::Centered => {
                        get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.)
                            * offset_transform
                    }
                },
                // Determine default visibility
                match &layer.visible {
                    true => Visibility::Inherited,
                    false => Visibility::Hidden,
                },
            ))
            .set_parent(map_entity)
            .id();

        let layer_infos = TiledLayerCreated {
            map: map_entity,
            layer: layer_entity,
            map_handle: map_handle.clone(),
            layer_id,
        };

        match layer.layer_type() {
            LayerType::Tiles(tile_layer) => {
                commands.entity(layer_entity).insert((
                    Name::new(format!("TiledMapTileLayer({})", layer.name)),
                    TiledMapTileLayer,
                ));
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
                commands.entity(layer_entity).insert((
                    Name::new(format!("TiledMapObjectLayer({})", layer.name)),
                    TiledMapObjectLayer,
                ));
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
                commands.entity(layer_entity).insert((
                    Name::new(format!("TiledMapGroupLayer({})", layer.name)),
                    TiledMapGroupLayer,
                ));
                warn!("Group layers are not yet implemented");
            }
            LayerType::Image(image_layer) => {
                commands.entity(layer_entity).insert((
                    Name::new(format!("TiledMapImageLayer({})", layer.name)),
                    TiledMapImageLayer,
                ));
                load_image_layer(commands, tiled_map, &layer_infos, image_layer, asset_server);
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
    _render_settings: &TilemapRenderSettings,
    entity_map: &mut HashMap<(String, TileId), Vec<Entity>>,
    event_list: &mut Vec<TiledSpecialTileCreated>,
) {
    // The TilemapBundle requires that all tile images come exclusively from a single
    // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
    // the per-tile images must be the same size. Since Tiled allows tiles of mixed
    // tilesets on each layer and allows differently-sized tile images in each tileset,
    // this means we need to load each combination of tileset and layer separately.
    for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
        let Some((usable_for_tiles_layer, tilemap_texture)) =
            tiled_map.tilemap_textures.get(&tileset_index)
        else {
            log::warn!("Skipped creating layer with missing tilemap textures.");
            continue;
        };

        if !usable_for_tiles_layer {
            continue;
        }

        let grid_size = get_grid_size(&tiled_map.map);
        let mut _map_size = get_map_size(&tiled_map.map);
        let mut _offset_x = 0.;
        let mut _offset_y = 0.;

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

        let _tile_storage = match tile_layer {
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
                _map_size = new_map_size;
                // log::info!("Infinite layer origin: {:?}", origin);
                _offset_x += origin.0 * grid_size.x;
                _offset_y -= origin.1 * grid_size.y;
                storage
            }
        };

        #[cfg(feature = "render")]
        commands
            .entity(layer_for_tileset_entity)
            .insert(TilemapBundle {
                grid_size,
                size: _map_size,
                storage: _tile_storage,
                texture: tilemap_texture.clone(),
                tile_size: TilemapTileSize {
                    x: tileset.tile_width as f32,
                    y: tileset.tile_height as f32,
                },
                spacing: TilemapSpacing {
                    x: tileset.spacing as f32,
                    y: tileset.spacing as f32,
                },
                transform: Transform::from_xyz(
                    _offset_x + grid_size.x / 2.,
                    -_offset_y + grid_size.y / 2.,
                    0.,
                ),
                map_type: get_map_type(&tiled_map.map),
                render_settings: *_render_settings,
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
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(layer_for_tileset_entity),
                        texture_index: TileTextureIndex(texture_index),
                        flip: TileFlip {
                            x: layer_tile_data.flip_h,
                            y: layer_tile_data.flip_v,
                            d: layer_tile_data.flip_d,
                        },
                        ..Default::default()
                    },
                    Name::new(format!("TiledMapTile({},{})", tile_pos.x, tile_pos.y)),
                    TiledMapTile,
                ))
                .set_parent(layer_for_tileset_entity)
                .id();

            handle_special_tile(
                commands,
                TiledSpecialTileCreated::from_layer(
                    layer_infos,
                    layer_for_tileset_entity,
                    tile_entity,
                    IVec2::new(mapped_x, mapped_y),
                    tile_pos,
                ),
                &tile,
                layer_tile.id(),
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
    _tiled_map: &TiledMap,
    layer_infos: &TiledLayerCreated,
    layer_for_tileset_entity: Entity,
    infinite_layer: &InfiniteTileLayer,
    tileset_index: usize,
    tilemap_texture: &TilemapTexture,
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
                        *_tiled_map.tile_image_offsets.get(&(tileset_index, layer_tile.id()))
                            .expect("The offset into to image vector should have been saved during the initial load."),
                    #[cfg(not(feature = "atlas"))]
                    _ => unreachable!()
                };

                let tile_pos = TilePos {
                    x: tile_x as u32,
                    y: map_size.y - 1 - tile_y as u32,
                };
                let tile_entity = commands
                    .spawn((
                        TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(layer_for_tileset_entity),
                            texture_index: TileTextureIndex(texture_index),
                            flip: TileFlip {
                                x: layer_tile_data.flip_h,
                                y: layer_tile_data.flip_v,
                                d: layer_tile_data.flip_d,
                            },
                            ..Default::default()
                        },
                        Name::new(format!("Tile({},{})", tile_pos.x, tile_pos.y)),
                        TiledMapTile,
                    ))
                    .set_parent(layer_for_tileset_entity)
                    .id();
                handle_special_tile(
                    commands,
                    TiledSpecialTileCreated::from_layer(
                        layer_infos,
                        layer_for_tileset_entity,
                        tile_entity,
                        IVec2::new(
                            chunk_pos.0 * ChunkData::WIDTH as i32 + x as i32,
                            chunk_pos.1 * ChunkData::HEIGHT as i32 + y as i32,
                        ),
                        tile_pos,
                    ),
                    &tile,
                    layer_tile.id(),
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
            .spawn((
                Name::new(format!("Object({})", object_data.name)),
                TiledMapObject,
                Transform::from_xyz(object_position.x, object_position.y, 0.),
            ))
            .set_parent(layer_infos.layer)
            .id();

        let mut sprite = None;
        let mut animation = None;

        if let Some(tile) = object_data.get_tile() {
            match tile.tileset_location() {
                TilesetLocation::Map(tileset_index) => {
                    sprite = tiled_map.tilemap_textures.get(tileset_index).and_then(|(_ , texture)| {
                        match texture {
                            TilemapTexture::Single(single) => {
                                tiled_map.texture_atlas_layout.get(tileset_index).map(|atlas| {
                                    Sprite {
                                        image: single.clone(),
                                        texture_atlas: Some(TextureAtlas {
                                            layout: atlas.clone(),
                                            index: tile.id() as usize,
                                        }),
                                        anchor: Anchor::BottomLeft,
                                        ..default()
                                    }
                                })
                            },
                            #[cfg(not(feature = "atlas"))]
                            TilemapTexture::Vector(vector) => {
                                let index = *tiled_map.tile_image_offsets.get(&(*tileset_index, tile.id()))
                                    .expect("The offset into to image vector should have been saved during the initial load.");
                                vector.get(index as usize).map(|image| {
                                    Sprite {
                                        image: image.clone(),
                                        anchor: Anchor::BottomLeft,
                                        ..default()
                                    }
                                })
                            }
                            #[cfg(not(feature = "atlas"))]
                            _ => unreachable!(),
                        }
                    });
                    animation =
                        tile.get_tile()
                            .and_then(|t| get_animated_tile(&t))
                            .map(|animation| TiledAnimation {
                                start: animation.start as usize,
                                end: animation.end as usize,
                                timer: Timer::from_seconds(
                                    1. / (animation.speed
                                        * (animation.end - animation.start) as f32),
                                    TimerMode::Repeating,
                                ),
                            });
                }
                TilesetLocation::Template(_) => {
                    log::warn!("Objects from template are not yet supported");
                }
            }
        }

        match (sprite, animation) {
            (Some(sprite), None) => {
                commands.entity(object_entity).insert(sprite);
            }
            (Some(sprite), Some(animation)) => {
                commands.entity(object_entity).insert((sprite, animation));
            }
            _ => {}
        }

        entity_map.insert(object_data.id(), object_entity);
        event_list.push(TiledObjectCreated::from_layer(
            layer_infos,
            object_entity,
            object_id,
        ));
    }
}

fn load_image_layer(
    commands: &mut Commands,
    tiled_map: &TiledMap,
    layer_infos: &TiledLayerCreated,
    image_layer: ImageLayer,
    asset_server: &Res<AssetServer>,
) {
    let map_type = get_map_type(&tiled_map.map);
    let grid_size = get_grid_size(&tiled_map.map);
    let map_size = get_map_size(&tiled_map.map);

    if let Some(image) = &image_layer.image {
        let image_position =
            from_tiled_coords_to_bevy(Vec2::splat(0.), &map_type, &map_size, &grid_size);
        commands
            .spawn((
                Name::new(format!("Image({})", image.source.display())),
                TiledMapImage,
                Sprite {
                    image: asset_server.load(image.source.clone()),
                    ..Default::default()
                },
                Transform::from_xyz(
                    image_position.x + image.width as f32 / 2.,
                    image_position.y - image.height as f32 / 2.,
                    0.,
                ),
            ))
            .set_parent(layer_infos.layer);
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

    // duration is in ms and we want a 'frames per second' speed
    Some(AnimatedTile {
        start: first_tile.tile_id,
        end: last_tile.tile_id + 1,
        speed: 1000. / (first_tile.duration * (last_tile.tile_id - first_tile.tile_id + 1)) as f32,
    })
}

fn handle_special_tile(
    commands: &mut Commands,
    tile_infos: TiledSpecialTileCreated,
    tile: &Tile,
    tile_id: TileId,
    entity_map: &mut HashMap<(String, TileId), Vec<Entity>>,
    event_list: &mut Vec<TiledSpecialTileCreated>,
) {
    let mut is_special_tile = false;

    // Handle animated tiles
    if let Some(animated_tile) = get_animated_tile(tile) {
        commands.entity(tile_infos.tile).insert(animated_tile);
    }

    // Handle custom tiles (with user properties)
    if !tile.properties.is_empty() {
        let key = (tile.tileset().name.clone(), tile_id);
        entity_map
            .entry(key)
            .and_modify(|entities| {
                entities.push(tile_infos.tile);
            })
            .or_insert(vec![tile_infos.tile]);
        is_special_tile = true;
    }

    // Handle tiles with collision
    if let Some(_collision) = tile.collision.as_ref() {
        is_special_tile = true;
    }

    if is_special_tile {
        event_list.push(tile_infos);
    }
}
