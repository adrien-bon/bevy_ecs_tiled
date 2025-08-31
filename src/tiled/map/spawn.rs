//! Systems and utilities for spawning Tiled map entities.
//!
//! This module contains logic for instantiating Bevy entities and components from Tiled map data.
//! It handles the creation of map layers, tiles, objects, and their associated components in the ECS world,
//! enabling the rendering and interaction of Tiled maps within a Bevy application.

use crate::{prelude::*, tiled::event::TiledEventWriters, tiled::layer::TiledLayerParallax};
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_tilemap::prelude::{
    AnimatedTile, IsoCoordSystem, TileBundle, TileFlip, TileStorage, TileTextureIndex, TilemapId,
    TilemapTexture,
};
use tiled::{ImageLayer, LayerType, ObjectLayer, TilesetLocation};

#[cfg(feature = "render")]
use bevy_ecs_tilemap::prelude::{TilemapBundle, TilemapSpacing};

#[cfg(feature = "user_properties")]
use crate::tiled::properties::command::PropertiesCommandExt;

use super::loader::tileset_path;

pub(crate) fn spawn_map(
    commands: &mut Commands,
    map_entity: Entity,
    map_asset_id: AssetId<TiledMapAsset>,
    tiled_map: &TiledMapAsset,
    map_storage: &mut TiledMapStorage,
    render_settings: &TilemapRenderSettings,
    layer_offset: &TiledMapLayerZOffset,
    asset_server: &Res<AssetServer>,
    event_writers: &mut TiledEventWriters,
    anchor: &TilemapAnchor,
) {
    commands.entity(map_entity).insert(Name::new(format!(
        "TiledMap: {}",
        tiled_map.map.source.display()
    )));

    let map_event = TiledEvent::new(map_entity, MapCreated)
        .with_map(map_entity, map_asset_id)
        .to_owned();

    let mut layer_events: Vec<TiledEvent<LayerCreated>> = Vec::new();
    let mut object_events: Vec<TiledEvent<ObjectCreated>> = Vec::new();
    let mut tilemap_events: Vec<TiledEvent<TilemapCreated>> = Vec::new();
    let mut tile_events: Vec<TiledEvent<TileCreated>> = Vec::new();

    // Order of the differents layers in the .TMX file is important:
    // a layer appearing last in the .TMX should appear above previous layers
    // Start with a negative offset so in the end we end up with the top layer at Z-offset from settings
    let mut offset_z = tiled_map.map.layers().len() as f32 * (-layer_offset.0);

    // Once materials have been created/added we need to then create the layers.
    for (layer_id, layer) in tiled_map.map.layers().enumerate() {
        let layer_id = layer_id as u32;
        // Increment Z offset and compute layer transform offset
        offset_z += layer_offset.0;
        let offset_transform = Transform::from_xyz(layer.offset_x, -layer.offset_y, offset_z);

        // Spawn layer entity and attach it to the map entity
        let layer_entity = commands
            .spawn((
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

        let layer_event = map_event
            .transmute(Some(layer_entity), LayerCreated)
            .with_layer(layer_entity, layer_id)
            .to_owned();

        // Add parallax component if the layer has parallax values
        let has_parallax = layer.parallax_x != 1.0 || layer.parallax_y != 1.0;
        let layer_position = tiled_map
            .world_space_from_tiled_position(anchor, Vec2::new(layer.offset_x, layer.offset_y));

        // Apply parallax to the layer entity if needed (works for all layer types)
        if has_parallax {
            commands.entity(layer_entity).insert(TiledLayerParallax {
                parallax_x: layer.parallax_x,
                parallax_y: layer.parallax_y,
                base_position: layer_position,
            });
        }

        match layer.layer_type() {
            LayerType::Tiles(tile_layer) => {
                commands.entity(layer_entity).insert((
                    Name::new(format!("TiledMapTileLayer({})", layer.name)),
                    TiledLayer::Tiles,
                ));
                spawn_tiles_layer(
                    commands,
                    tiled_map,
                    &layer_event,
                    layer,
                    tile_layer,
                    render_settings,
                    &mut map_storage.tiles,
                    &mut tilemap_events,
                    &mut tile_events,
                    anchor,
                );
            }
            LayerType::Objects(object_layer) => {
                commands.entity(layer_entity).insert((
                    Name::new(format!("TiledMapObjectLayer({})", layer.name)),
                    TiledLayer::Objects,
                ));
                spawn_objects_layer(
                    commands,
                    tiled_map,
                    &layer_event,
                    object_layer,
                    &mut map_storage.objects,
                    &mut object_events,
                    anchor,
                );
            }
            LayerType::Group(_group_layer) => {
                commands.entity(layer_entity).insert((
                    Name::new(format!("TiledMapGroupLayer({})", layer.name)),
                    TiledLayer::Group,
                ));
                warn!("Group layers are not yet implemented");
            }
            LayerType::Image(image_layer) => {
                commands.entity(layer_entity).insert((
                    Name::new(format!("TiledMapImageLayer({})", layer.name)),
                    TiledLayer::Image,
                ));
                spawn_image_layer(
                    commands,
                    tiled_map,
                    &layer_event,
                    image_layer,
                    asset_server,
                    anchor,
                );
            }
        };

        map_storage.layers.insert(layer.id(), layer_entity);
        layer_events.push(layer_event);
    }

    #[cfg(feature = "user_properties")]
    {
        let mut props = tiled_map.properties.clone().hydrate(&map_storage.objects);

        commands.entity(map_entity).insert_properties(props.map);

        for (id, &entity) in map_storage.objects.iter() {
            commands
                .entity(entity)
                .insert_properties(props.objects.remove(id).unwrap());
        }

        for (id, &entity) in map_storage.layers.iter() {
            commands
                .entity(entity)
                .insert_properties(props.layers.remove(id).unwrap());
        }

        for (id, entities) in map_storage.tiles.iter() {
            let Some(p) = props.tiles.get(&id.0).and_then(|e| e.get(&id.1)) else {
                continue;
            };
            for &entity in entities {
                commands.entity(entity).insert_properties(p.clone());
            }
        }
    }

    // Send events and trigger observers
    map_event.send(commands, &mut event_writers.map_created);

    for e in layer_events {
        e.send(commands, &mut event_writers.layer_created);
    }
    for e in tilemap_events {
        e.send(commands, &mut event_writers.tilemap_created);
    }
    for e in tile_events {
        e.send(commands, &mut event_writers.tile_created);
    }
    for e in object_events {
        e.send(commands, &mut event_writers.object_created);
    }
}

fn spawn_tiles_layer(
    commands: &mut Commands,
    tiled_map: &TiledMapAsset,
    layer_event: &TiledEvent<LayerCreated>,
    layer: Layer,
    tiles_layer: TileLayer,
    _render_settings: &TilemapRenderSettings,
    entity_map: &mut HashMap<(u32, TileId), Vec<Entity>>,
    tilemap_events: &mut Vec<TiledEvent<TilemapCreated>>,
    tile_events: &mut Vec<TiledEvent<TileCreated>>,
    _anchor: &TilemapAnchor,
) {
    // The `TilemapBundle` requires that all tile images come exclusively from a single
    // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
    // the per-tile images must be the same size. Since Tiled allows tiles of mixed
    // tilesets on each layer and allows differently-sized tile images in each tileset,
    // this means we need to load each combination of tileset and layer separately.
    for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
        let tileset_index = tileset_index as u32;
        let Some(path) = tiled_map.tilesets_path_by_index.get(&tileset_index) else {
            log::warn!("Skipped creating layer with missing tilemap textures (index {tileset_index} not found).");
            continue;
        };

        let Some(t) = tiled_map.tilesets.get(path) else {
            log::warn!(
                "Skipped creating layer with missing tilemap textures (path {path:?} not found)."
            );
            continue;
        };

        if !t.usable_for_tiles_layer {
            continue;
        }

        let tilemap_entity = commands
            .spawn((
                Name::new(format!("TiledTilemap({}, {})", layer.name, tileset.name)),
                TiledTilemap,
                ChildOf(layer_event.origin),
            ))
            .id();

        let tilemap_event = layer_event
            .transmute(Some(tilemap_entity), TilemapCreated)
            .with_tilemap(tilemap_entity, tileset_index)
            .to_owned();
        tilemap_events.push(tilemap_event);

        let _tile_storage = spawn_tiles(
            commands,
            tiled_map,
            &tilemap_event,
            tilemap_entity,
            &t.tilemap_texture,
            tileset_index,
            &tiles_layer,
            entity_map,
            tile_events,
        );

        #[cfg(feature = "render")]
        {
            let grid_size = grid_size_from_map(&tiled_map.map);
            commands.entity(tilemap_entity).insert(TilemapBundle {
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
                transform: Transform::from_xyz(
                    tileset.offset_x as f32,
                    -tileset.offset_y as f32,
                    0.0,
                ),
                map_type: tilemap_type_from_map(&tiled_map.map),
                render_settings: *_render_settings,
                anchor: *_anchor,
                ..default()
            });
        }
    }
}

fn spawn_tiles(
    commands: &mut Commands,
    tiled_map: &TiledMapAsset,
    layer_event: &TiledEvent<TilemapCreated>,
    layer_entity: Entity,
    tilemap_texture: &TilemapTexture,
    tileset_id: u32,
    tiles_layer: &TileLayer,
    entity_map: &mut HashMap<(u32, TileId), Vec<Entity>>,
    tile_events: &mut Vec<TiledEvent<TileCreated>>,
) -> TileStorage {
    let tilemap_size = tiled_map.tilemap_size;
    let mut tile_storage = TileStorage::empty(tilemap_size);
    tiled_map.for_each_tile(
        tiles_layer,
        |layer_tile, layer_tile_data, tile_pos, _| {
            let Some(tile) = layer_tile.get_tile() else {
                return;
            };

            if tileset_id as usize != layer_tile.tileset_index() {
                return;
            }

            #[cfg(not(feature = "atlas"))]
            let Some(path) = tiled_map.tilesets_path_by_index.get(&tileset_id) else {
                return;
            };

            let texture_index = match tilemap_texture {
                TilemapTexture::Single(_) => layer_tile.id(),
                #[cfg(not(feature = "atlas"))]
                TilemapTexture::Vector(_) => *tiled_map
                    .tilesets
                    .get(path)
                    .and_then(|t| t.tile_image_offsets.get(&layer_tile.id()))
                    .expect(
                        "The offset into the image vector for tilemap should have been saved during the initial load.",
                    ),
                #[cfg(not(feature = "atlas"))]
                _ => unreachable!(),
            };
            let tile_entity = commands
                .spawn((
                    Name::new(format!("TiledMapTile({},{})", tile_pos.x, tile_pos.y)),
                    TiledTile,
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(layer_entity),
                        texture_index: TileTextureIndex(texture_index),
                        flip: TileFlip {
                            x: layer_tile_data.flip_h,
                            y: layer_tile_data.flip_v,
                            d: layer_tile_data.flip_d,
                        },
                        ..default()
                    },
                    ChildOf(layer_entity),
                ))
                .id();

            // Handle animated tiles
            if let Some(animated_tile) = get_animated_tile(&tile) {
                commands.entity(tile_entity).insert(animated_tile);
            }

            let tile_id = layer_tile.id();

            // Handle custom tiles (with user properties)
            if !tile.properties.is_empty() {
                let tile_event = layer_event.transmute(Some(tile_entity), TileCreated).with_tile(
                    tile_entity,
                    tile_pos,
                    tile_id,
                ).to_owned();
                tile_events.push(tile_event);
            }

            // Update map storage with tile entity
            let key = (tileset_id, tile_id);
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

fn spawn_objects_layer(
    commands: &mut Commands,
    tiled_map: &TiledMapAsset,
    layer_event: &TiledEvent<LayerCreated>,
    object_layer: ObjectLayer,
    entity_map: &mut HashMap<u32, Entity>,
    object_events: &mut Vec<TiledEvent<ObjectCreated>>,
    anchor: &TilemapAnchor,
) {
    for (index, object_data) in object_layer.objects().enumerate() {
        let tiled_object = TiledObject::from_object_data(&object_data);
        let mut pos = tiled_map.object_relative_position(&object_data, anchor);

        // For isometric maps, we need to adjust the position of tile objects
        // to match the isometric grid.
        if matches!(
            tilemap_type_from_map(&tiled_map.map),
            TilemapType::Isometric(..)
        ) {
            if let TiledObject::Tile { width, height: _ } = tiled_object {
                pos.x -= width / 2.;
            }
        }

        let transform = Transform::from_isometry(
            Isometry3d::from_translation(pos.extend(index as f32 * 0.001))
                * Isometry3d::from_rotation(Quat::from_rotation_z(f32::to_radians(
                    -object_data.rotation,
                ))),
        );

        let object_kind = match tiled_object {
            TiledObject::Point => "Point",
            TiledObject::Tile { .. } => "Tile",
            TiledObject::Text => "Text",
            TiledObject::Rectangle { .. } => "Rectangle",
            TiledObject::Ellipse { .. } => "Ellipse",
            TiledObject::Polygon { .. } => "Polygon",
            TiledObject::Polyline { .. } => "Polyline",
        };

        let object_entity = commands
            .spawn((
                Name::new(format!("{object_kind}({})", object_data.name)),
                ChildOf(layer_event.origin),
                tiled_object,
                transform,
                match &object_data.visible {
                    true => Visibility::Inherited,
                    false => Visibility::Hidden,
                },
            ))
            .id();

        // Handle objects containing tile data:
        // we want to add a Sprite component to the object entity
        // and possibly an animation component if the tile is animated.
        match handle_tile_object(&object_data, tiled_map) {
            (Some((sprite, offset_transform)), None) => {
                commands.spawn((
                    Name::new("TileVisual"),
                    ChildOf(object_entity),
                    sprite,
                    offset_transform,
                ));
            }
            (Some((sprite, offset_transform)), Some(animation)) => {
                commands.spawn((
                    Name::new("TileVisual"),
                    ChildOf(object_entity),
                    sprite,
                    offset_transform,
                    animation,
                ));
            }
            _ => {}
        };

        entity_map.insert(object_data.id(), object_entity);
        let object_event = layer_event
            .transmute(Some(object_entity), ObjectCreated)
            .with_object(object_entity, object_data.id())
            .to_owned();
        object_events.push(object_event);
    }
}

fn handle_tile_object(
    object: &Object,
    tiled_map: &TiledMapAsset,
) -> (Option<(Sprite, Transform)>, Option<TiledAnimation>) {
    let Some(tile) = (*object).get_tile() else {
        return (None, None);
    };

    // Assume tile objets always have a rectangular shape
    let tiled::ObjectShape::Rect { width, height } = object.shape else {
        return (None, None);
    };

    let path = match tile.tileset_location() {
        TilesetLocation::Map(tileset_index) => {
            let tileset_index = *tileset_index as u32;
            tiled_map
                .tilesets_path_by_index
                .get(&tileset_index)
                .expect("Cannot find tileset path for object tile")
        }
        TilesetLocation::Template(tileset) => &tileset_path(tileset)
            .expect("Cannot find object tile from Template")
            .to_owned(),
    };

    let Some(transform) = tile.get_tile().map(|t| {
        let unscaled_tile_size = match &t.image {
            Some(image) => {
                // tile is in image collection
                Vec2::new(image.width as f32, image.height as f32)
            }
            None => Vec2::new(
                t.tileset().tile_width as f32,
                t.tileset().tile_height as f32,
            ),
        };
        let scale = Vec2::new(width, height) / unscaled_tile_size;
        Transform::from_xyz(
            t.tileset().offset_x as f32 * scale.x,
            -t.tileset().offset_y as f32 * scale.y,
            0.0,
        )
    }) else {
        return (None, None);
    };

    let Some(sprite) = tiled_map.tilesets.get(path).and_then(|t| {
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
                        flip_x: tile.flip_h,
                        flip_y: tile.flip_v,
                        custom_size: Some(Vec2::new(
                            width,
                            height
                        )),
                        ..default()
                    }
                })
            },
            #[cfg(not(feature = "atlas"))]
            TilemapTexture::Vector(vector) => {
                let index = *t.tile_image_offsets.get(&tile.id())
                    .expect("The offset into the image vector for template should have been saved during the initial load.");
                vector.get(index as usize).map(|image| {
                    Sprite {
                        image: image.clone(),
                        anchor: Anchor::BottomLeft,
                        flip_x: tile.flip_h,
                        flip_y: tile.flip_v,
                        custom_size: Some(Vec2::new(
                            width,
                            height
                        )),
                        ..default()
                    }
                })
            }
            #[cfg(not(feature = "atlas"))]
            _ => unreachable!(),
        }
    }) else {
        return (None, None);
    };

    // Handle the case of an animated tile
    let animation = tile
        .get_tile()
        .and_then(|t| get_animated_tile(&t))
        .map(|animation| TiledAnimation {
            start: animation.start as usize,
            end: animation.end as usize,
            timer: Timer::from_seconds(
                1. / (animation.speed * (animation.end - animation.start) as f32),
                TimerMode::Repeating,
            ),
        });

    (Some((sprite, transform)), animation)
}

fn spawn_image_layer(
    commands: &mut Commands,
    tiled_map: &TiledMapAsset,
    layer_event: &TiledEvent<LayerCreated>,
    image_layer: ImageLayer,
    asset_server: &Res<AssetServer>,
    anchor: &TilemapAnchor,
) {
    if let Some(image) = &image_layer.image {
        let image_position = tiled_map.world_space_from_tiled_position(
            anchor,
            match tilemap_type_from_map(&tiled_map.map) {
                // Special case for isometric maps where image origin
                // is not (0, 0) but (-map_width, +map_height)
                TilemapType::Isometric(IsoCoordSystem::Diamond) => {
                    let grid_size = grid_size_from_map(&tiled_map.map);
                    let map_size = tiled_map.tilemap_size;
                    Vec2 {
                        x: map_size.x as f32 * grid_size.y / -2.,
                        y: map_size.y as f32 * grid_size.y / 2.,
                    }
                }
                _ => Vec2::ZERO,
            },
        );
        commands.spawn((
            Name::new(format!("Image({})", image.source.display())),
            TiledImage,
            ChildOf(layer_event.origin),
            Sprite {
                image: asset_server.load(image.source.clone()),
                anchor: Anchor::TopLeft,
                ..default()
            },
            Transform::from_translation(image_position.extend(0.)),
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
