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
use bevy::{platform::collections::HashMap, prelude::*, sprite::Anchor};
use bevy_ecs_tilemap::prelude::*;
use tiled::{ImageLayer, Layer, LayerType, ObjectLayer, Tile, TileId, TileLayer, TilesetLocation};

#[allow(clippy::too_many_arguments)]
pub(crate) fn load_map(
    commands: &mut Commands,
    map_entity: Entity,
    map_asset_id: AssetId<TiledMap>,
    tiled_map: &TiledMap,
    tiled_id_storage: &mut TiledMapStorage,
    render_settings: &TilemapRenderSettings,
    layer_offset: &TiledMapLayerZOffset,
    asset_server: &Res<AssetServer>,
    event_writers: &mut TiledMapEventWriters,
    anchor: &TilemapAnchor,
) {
    commands.entity(map_entity).insert((
        Name::new(format!("TiledMap: {}", tiled_map.map.source.display())),
        TiledMapMarker,
    ));

    let map_event = TiledMapCreated {
        entity: map_entity,
        asset_id: map_asset_id,
    };

    let mut layer_events: Vec<TiledLayerCreated> = Vec::new();
    let mut object_events: Vec<TiledObjectCreated> = Vec::new();
    let mut special_tile_events: Vec<TiledTileCreated> = Vec::new();

    // Order of the differents layers in the .TMX file is important:
    // a layer appearing last in the .TMX should appear above previous layers
    // Start with a negative offset so in the end we end up with the top layer at Z-offset from settings
    let mut offset_z = tiled_map.map.layers().len() as f32 * (-layer_offset.0);

    // Once materials have been created/added we need to then create the layers.
    for (layer_id, layer) in tiled_map.map.layers().enumerate() {
        // Increment Z offset and compute layer transform offset
        offset_z += layer_offset.0;
        let offset_transform = Transform::from_xyz(layer.offset_x, -layer.offset_y, offset_z);

        // Spawn layer entity and attach it to the map entity
        let layer_entity = commands
            .spawn((
                TiledMapLayer,
                ChildOf(map_entity),
                // Apply layer Transform using both layer base Transform and Tiled offset
                offset_transform,
                // Determine layer default visibility
                match &layer.visible {
                    true => Visibility::Inherited,
                    false => Visibility::Hidden,
                },
            ))
            .id();

        let layer_event = TiledLayerCreated {
            map: map_event,
            entity: layer_entity,
            id: layer_id,
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
                    &layer_event,
                    layer,
                    tile_layer,
                    render_settings,
                    &mut tiled_id_storage.tiles,
                    &mut special_tile_events,
                    anchor,
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
                    &layer_event,
                    object_layer,
                    &mut tiled_id_storage.objects,
                    &mut object_events,
                    anchor,
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
                let pos = from_tiled_position_to_world_space(
                    tiled_map,
                    anchor,
                    Vec2::new(layer.offset_x, layer.offset_y),
                );
                commands.entity(layer_entity).insert((
                    Name::new(format!("TiledMapImageLayer({})", layer.name)),
                    TiledMapImageLayer,
                    Transform::from_translation(pos.extend(offset_z)),
                ));
                load_image_layer(commands, tiled_map, &layer_event, image_layer, asset_server);
            }
        };

        tiled_id_storage.layers.insert(layer.id(), layer_entity);
        layer_events.push(layer_event);
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

    // Send events and trigger observers
    commands.trigger_targets(map_event, map_entity);
    event_writers.map_event.write(map_event);
    for e in layer_events {
        commands.trigger_targets(e, map_entity);
        event_writers.layer_event.write(e);
    }
    for e in object_events {
        commands.trigger_targets(e, map_entity);
        event_writers.object_event.write(e);
    }
    for e in special_tile_events {
        commands.trigger_targets(e, map_entity);
        event_writers.tile_event.write(e);
    }
}

#[allow(clippy::too_many_arguments)]
fn load_tiles_layer(
    commands: &mut Commands,
    tiled_map: &TiledMap,
    layer_event: &TiledLayerCreated,
    layer: Layer,
    tiles_layer: TileLayer,
    _render_settings: &TilemapRenderSettings,
    entity_map: &mut HashMap<(String, TileId), Vec<Entity>>,
    event_list: &mut Vec<TiledTileCreated>,
    _anchor: &TilemapAnchor,
) {
    // The TilemapBundle requires that all tile images come exclusively from a single
    // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
    // the per-tile images must be the same size. Since Tiled allows tiles of mixed
    // tilesets on each layer and allows differently-sized tile images in each tileset,
    // this means we need to load each combination of tileset and layer separately.
    for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
        let Some(t) = tiled_map.tilesets.get(&tileset_index) else {
            log::warn!("Skipped creating layer with missing tilemap textures.");
            continue;
        };

        if !t.usable_for_tiles_layer {
            continue;
        }

        let layer_for_tileset_entity = commands
            .spawn((
                Name::new(format!(
                    "TiledMapTileLayerForTileset({}, {})",
                    layer.name, tileset.name
                )),
                TiledMapTileLayerForTileset,
                ChildOf(layer_event.entity),
            ))
            .id();

        let _tile_storage = load_tiles(
            commands,
            tiled_map,
            layer_event,
            layer_for_tileset_entity,
            &t.tilemap_texture,
            tileset_index,
            &tiles_layer,
            entity_map,
            event_list,
        );

        #[cfg(feature = "render")]
        {
            let grid_size = get_grid_size(&tiled_map.map);
            commands
                .entity(layer_for_tileset_entity)
                .insert(TilemapBundle {
                    grid_size,
                    size: tiled_map.tilemap_size,
                    storage: _tile_storage,
                    texture: t.tilemap_texture.clone(),
                    tile_size: TilemapTileSize {
                        x: tileset.tile_width as f32,
                        y: tileset.tile_height as f32,
                    },
                    spacing: TilemapSpacing {
                        x: tileset.spacing as f32,
                        y: tileset.spacing as f32,
                    },
                    map_type: get_map_type(&tiled_map.map),
                    render_settings: *_render_settings,
                    anchor: *_anchor,
                    ..default()
                });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn load_tiles(
    commands: &mut Commands,
    tiled_map: &TiledMap,
    layer_event: &TiledLayerCreated,
    layer_for_tileset_entity: Entity,
    tilemap_texture: &TilemapTexture,
    tileset_index: usize,
    tiles_layer: &TileLayer,
    entity_map: &mut HashMap<(String, TileId), Vec<Entity>>,
    event_list: &mut Vec<TiledTileCreated>,
) -> TileStorage {
    let tilemap_size = tiled_map.tilemap_size;
    let mut tile_storage = TileStorage::empty(tilemap_size);
    for_each_tile(
        tiled_map,
        tiles_layer,
        |layer_tile, layer_tile_data, tile_pos, index| {
            let Some(tile) = layer_tile.get_tile() else {
                return;
            };
            if tileset_index != layer_tile.tileset_index() {
                return;
            }
            let texture_index = match tilemap_texture {
                TilemapTexture::Single(_) => layer_tile.id(),
                #[cfg(not(feature = "atlas"))]
                TilemapTexture::Vector(_) => *tiled_map
                    .tilesets
                    .get(&tileset_index)
                    .and_then(|t| t.tile_image_offsets.get(&layer_tile.id()))
                    .expect(
                        "The offset into to image vector should have been saved during the initial load.",
                    ),
                #[cfg(not(feature = "atlas"))]
                _ => unreachable!(),
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
                        ..default()
                    },
                    Name::new(format!("TiledMapTile({},{})", tile_pos.x, tile_pos.y)),
                    TiledMapTile,
                    ChildOf(layer_for_tileset_entity),
                ))
                .id();

            // Handle animated tiles
            if let Some(animated_tile) = get_animated_tile(&tile) {
                commands.entity(tile_entity).insert(animated_tile);
            }

            // Handle custom tiles (with user properties)
            if !tile.properties.is_empty() {
                event_list.push(TiledTileCreated {
                    layer: *layer_event,
                    parent: layer_for_tileset_entity,
                    entity: tile_entity,
                    index,
                    position: tile_pos,
                });
            }

            // Update map storage with tile entity
            let key = (tile.tileset().name.clone(), layer_tile.id());
            entity_map
                .entry(key)
                .and_modify(|entities| {
                    entities.push(tile_entity);
                })
                .or_insert(vec![tile_entity]);

            // Add our tile to the bevy_ecs_tilemap::TileStorage
            tile_storage.set(&tile_pos, tile_entity);
        },
    );
    tile_storage
}

fn load_objects_layer(
    commands: &mut Commands,
    tiled_map: &TiledMap,
    layer_event: &TiledLayerCreated,
    object_layer: ObjectLayer,
    entity_map: &mut HashMap<u32, Entity>,
    event_list: &mut Vec<TiledObjectCreated>,
    anchor: &TilemapAnchor,
) {
    for (object_id, object_data) in object_layer.objects().enumerate() {
        let object_position = from_tiled_position_to_world_space(
            tiled_map,
            anchor,
            Vec2::new(object_data.x, object_data.y),
        );
        let object_entity = commands
            .spawn((
                Name::new(format!("Object({})", object_data.name)),
                TiledMapObject,
                ChildOf(layer_event.entity),
                Transform::from_xyz(object_position.x, object_position.y, 0.),
                match &object_data.visible {
                    true => Visibility::Inherited,
                    false => Visibility::Hidden,
                },
            ))
            .id();

        let mut sprite = None;
        let mut animation = None;

        // Handle objects containing tile data: we want to add a Sprite component to the object with the tile image
        if let Some(tile) = object_data.get_tile() {
            match tile.tileset_location() {
                TilesetLocation::Map(tileset_index) => {
                    sprite = tiled_map.tilesets.get(tileset_index).and_then(|t| {
                        match &t.tilemap_texture {
                            TilemapTexture::Single(single) => {
                                t.texture_atlas_layout_handle.as_ref().map(|handle| {
                                    Sprite {
                                        image: single.clone(),
                                        texture_atlas: Some(TextureAtlas {
                                            layout: handle.clone(),
                                            index: tile.id() as usize,
                                        }),
                                        anchor: Anchor::BottomLeft,
                                        ..default()
                                    }
                                })
                            },
                            #[cfg(not(feature = "atlas"))]
                            TilemapTexture::Vector(vector) => {
                                let index = *t.tile_image_offsets.get(&tile.id())
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

                    // Handle the case of an animated tile
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
                commands.entity(object_entity).insert((
                    sprite,
                    if object_data.visible {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                ));
            }
            (Some(sprite), Some(animation)) => {
                commands.entity(object_entity).insert((
                    sprite,
                    animation,
                    if object_data.visible {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                ));
            }
            _ => {}
        };
        entity_map.insert(object_data.id(), object_entity);
        event_list.push(TiledObjectCreated {
            layer: *layer_event,
            entity: object_entity,
            id: object_id,
        });
    }
}

fn load_image_layer(
    commands: &mut Commands,
    tiled_map: &TiledMap,
    layer_event: &TiledLayerCreated,
    image_layer: ImageLayer,
    asset_server: &Res<AssetServer>,
) {
    if let Some(image) = &image_layer.image {
        let image_position = match get_map_type(&tiled_map.map) {
            // Special case for isometric maps where image origin is not (0, 0)
            TilemapType::Isometric(IsoCoordSystem::Diamond) => {
                let grid_size = get_grid_size(&tiled_map.map);
                let map_size = tiled_map.tilemap_size;
                Vec2 {
                    x: map_size.x as f32 * grid_size.y / -2.,
                    y: map_size.y as f32 * grid_size.y / 2.,
                }
            }
            _ => Vec2::ZERO,
        };
        commands.spawn((
            Name::new(format!("Image({})", image.source.display())),
            TiledMapImage,
            ChildOf(layer_event.entity),
            Sprite {
                image: asset_server.load(image.source.clone()),
                anchor: Anchor::TopLeft,
                ..default()
            },
            Transform::from_xyz(image_position.x, image_position.y, 0.),
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

    // duration is in ms and we want a 'frames per second' speed
    Some(AnimatedTile {
        start: first_tile.tile_id,
        end: last_tile.tile_id + 1,
        speed: 1000. / (first_tile.duration * (last_tile.tile_id - first_tile.tile_id + 1)) as f32,
    })
}
