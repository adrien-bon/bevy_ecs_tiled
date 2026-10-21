//! Systems and utilities for spawning Tiled map entities.
//!
//! This module contains logic for instantiating Bevy entities and components from Tiled map data.
//! It handles the creation of map layers, tiles, objects, and their associated components in the ECS world,
//! enabling the rendering and interaction of Tiled maps within a Bevy application.

use crate::{prelude::*, tiled::event::TiledMessageWriters};
#[cfg(feature = "text")]
use bevy::camera::primitives::Aabb;

use bevy::{prelude::*, sprite::Anchor};
#[cfg(feature = "render")]
use bevy_ecs_tilemap::prelude::{TilemapBundle, TilemapSpacing};

#[cfg(feature = "user_properties")]
use crate::tiled::properties::command::PropertiesCommandExt;

use super::loader::tileset_label;

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_map(
    commands: &mut Commands,
    map_entity: Entity,
    map_asset_id: AssetId<TiledMapAsset>,
    tiled_map: &TiledMapAsset,
    map_storage: &mut TiledMapStorage,
    render_settings: &TilemapRenderSettings,
    layer_z_offset: &TiledMapLayerZOffset,
    message_writers: &mut TiledMessageWriters,
    anchor: &TilemapAnchor,
) {
    commands.entity(map_entity).insert(Name::new(format!(
        "TiledMap: {}",
        tiled_map.map.source.display()
    )));

    let map_event = TiledEvent::new(map_entity, MapCreated)
        .with_map(map_entity, map_asset_id)
        .to_owned();

    let mut ctx = SpawnContext {
        commands,
        tiled_map,
        map_entity,
        map_storage,
        render_settings,
        anchor,
        layer_z_offset: layer_z_offset.0,
        // Order of the differents layers in the .TMX file is important:
        // a layer appearing last in the .TMX should appear above previous layers
        // Start with a negative offset so in the end we end up with the top layer at Z-offset from settings
        acc_z_offset: tiled_map.map.layers().len() as f32 * (-layer_z_offset.0),
        layer_events: Vec::new(),
        object_events: Vec::new(),
        tilemap_events: Vec::new(),
        tile_events: Vec::new(),
    };
    ctx.spawn_layers(tiled_map.map.layers(), map_entity, &map_event);
    #[cfg(feature = "user_properties")]
    ctx.insert_properties();

    let SpawnContext {
        commands,
        layer_events,
        object_events,
        tilemap_events,
        tile_events,
        ..
    } = ctx;

    // Send events and trigger observers
    map_event.send(commands, &mut message_writers.map_created);

    for e in layer_events {
        e.send(commands, &mut message_writers.layer_created);
    }
    for e in tilemap_events {
        e.send(commands, &mut message_writers.tilemap_created);
    }
    for e in tile_events {
        e.send(commands, &mut message_writers.tile_created);
    }
    for e in object_events {
        e.send(commands, &mut message_writers.object_created);
    }
}

/// State shared by all the functions spawning the entities of a single map.
struct SpawnContext<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    tiled_map: &'a TiledMapAsset,
    map_entity: Entity,
    map_storage: &'a mut TiledMapStorage,
    #[cfg_attr(not(feature = "render"), allow(dead_code))]
    render_settings: &'a TilemapRenderSettings,
    anchor: &'a TilemapAnchor,
    /// Z offset between two consecutive layers
    layer_z_offset: f32,
    /// Z offset of the last spawned layer
    acc_z_offset: f32,
    /// Events sent once the whole map has been spawned
    layer_events: Vec<TiledEvent<LayerCreated>>,
    object_events: Vec<TiledEvent<ObjectCreated>>,
    tilemap_events: Vec<TiledEvent<TilemapCreated>>,
    tile_events: Vec<TiledEvent<TileCreated>>,
}

impl SpawnContext<'_, '_, '_> {
    /// Insert user properties on the map and all its spawned layers, objects and tiles
    #[cfg(feature = "user_properties")]
    fn insert_properties(&mut self) {
        let commands = &mut *self.commands;
        let map_storage = &*self.map_storage;
        let mut props = self
            .tiled_map
            .properties
            .clone()
            .hydrate(&map_storage.objects);

        commands
            .entity(self.map_entity)
            .insert_properties(props.map);

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

    fn spawn_layers<'l>(
        &mut self,
        layers: impl ExactSizeIterator<Item = tiled::Layer<'l>>,
        parent_entity: Entity,
        map_event: &TiledEvent<MapCreated>,
    ) {
        for layer in layers {
            let layer_id = layer.id();
            let layer_position = Vec2::new(layer.offset_x, -layer.offset_y);

            // Increment Z offset and compute layer transform offset
            self.acc_z_offset += self.layer_z_offset;
            let layer_transform =
                Transform::from_translation(layer_position.extend(self.acc_z_offset));

            let commands = &mut *self.commands;

            // Spawn layer entity and attach it to the map entity
            let layer_entity = commands
                .spawn((
                    ChildOf(parent_entity),
                    TiledMapReference(self.map_entity),
                    TiledName(layer.name.clone()),
                    TiledLayerId(layer.id()),
                    // Apply layer Transform using both layer base Transform and Tiled offset
                    layer_transform,
                    // Determine layer default visibility
                    match &layer.visible {
                        true => Visibility::Inherited,
                        false => Visibility::Hidden,
                    },
                ))
                .id();

            // Add parallax component if the layer has parallax values
            if layer.parallax_x != 1.0 || layer.parallax_y != 1.0 {
                commands.entity(layer_entity).insert(TiledLayerParallax {
                    parallax_x: layer.parallax_x,
                    parallax_y: layer.parallax_y,
                    base_position: layer_position,
                });
            }

            let layer_event = map_event
                .transmute(Some(layer_entity), LayerCreated)
                .with_layer(layer_entity, layer_id)
                .to_owned();

            match layer.layer_type() {
                tiled::LayerType::Tiles(tile_layer) => {
                    self.commands.entity(layer_entity).insert((
                        Name::new(format!("TiledMapTileLayer({})", layer.name)),
                        TiledLayer::Tiles,
                    ));
                    self.spawn_tiles_layer(&layer_event, layer, tile_layer);
                }
                tiled::LayerType::Objects(object_layer) => {
                    self.commands.entity(layer_entity).insert((
                        Name::new(format!("TiledMapObjectLayer({})", layer.name)),
                        TiledLayer::Objects,
                    ));
                    self.spawn_objects_layer(&layer_event, object_layer);
                }
                tiled::LayerType::Group(group_layer) => {
                    self.commands.entity(layer_entity).insert((
                        Name::new(format!("TiledMapGroupLayer({})", layer.name)),
                        TiledLayer::Group,
                    ));
                    self.spawn_layers(group_layer.layers(), layer_entity, map_event);
                }
                tiled::LayerType::Image(image_layer) => {
                    self.commands.entity(layer_entity).insert((
                        Name::new(format!("TiledMapImageLayer({})", layer.name)),
                        TiledLayer::Image,
                    ));
                    self.spawn_image_layer(&layer_event, image_layer);
                }
            };

            self.map_storage.layers.insert(layer.id(), layer_entity);
            self.layer_events.push(layer_event);
        }
    }

    fn spawn_tiles_layer(
        &mut self,
        layer_event: &TiledEvent<LayerCreated>,
        layer: tiled::Layer,
        tiles_layer: tiled::TileLayer,
    ) {
        let tiled_map = self.tiled_map;
        // The `TilemapBundle` requires that all tile images come exclusively from a single
        // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
        // the per-tile images must be the same size. Since Tiled allows tiles of mixed
        // tilesets on each layer and allows differently-sized tile images in each tileset,
        // this means we need to load each combination of tileset and layer separately.
        for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
            let tileset_index = tileset_index as u32;
            let Some(label) = tiled_map.tilesets_label_by_index.get(&tileset_index) else {
                log::warn!("Skipped layer with missing tileset (index: {tileset_index})");
                continue;
            };

            let Some(t) = tiled_map.tilesets.get(label) else {
                log::warn!("Skipped layer with missing tileset (label: {label:?})");
                continue;
            };

            if !t.usable_for_tiles_layer {
                continue;
            }

            let tilemap_entity = self
                .commands
                .spawn((
                    Name::new(format!("TiledTilemap({}, {})", layer.name, tileset.name)),
                    TiledMapReference(layer_event.map_entity().unwrap()),
                    TiledTilemap,
                    TiledTilesetId(tileset_index),
                    TiledName(tileset.name.clone()),
                    ChildOf(layer_event.origin),
                ))
                .id();

            let tilemap_event = layer_event
                .transmute(Some(tilemap_entity), TilemapCreated)
                .with_tilemap(tilemap_entity, tileset_index)
                .to_owned();
            self.tilemap_events.push(tilemap_event);

            let _tile_storage = self.spawn_tiles(
                &tilemap_event,
                tilemap_entity,
                &t.tilemap_texture,
                tileset_index,
                &tiles_layer,
            );

            #[cfg(feature = "render")]
            {
                let grid_size = grid_size_from_map(&tiled_map.map);
                self.commands.entity(tilemap_entity).insert(TilemapBundle {
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
                    transform: Transform::from_translation(
                        crate::tiled::helpers::tilemap_offset(&tiled_map.map, tileset).extend(0.0),
                    ),
                    map_type: tilemap_type_from_map(&tiled_map.map),
                    render_settings: *self.render_settings,
                    anchor: *self.anchor,
                    ..default()
                });
            }
        }
    }

    fn spawn_tiles(
        &mut self,
        layer_event: &TiledEvent<TilemapCreated>,
        tilemap_entity: Entity,
        tilemap_texture: &TilemapTexture,
        tileset_id: u32,
        tiles_layer: &tiled::TileLayer,
    ) -> TileStorage {
        let tiled_map = self.tiled_map;
        let commands = &mut *self.commands;
        let entity_map = &mut self.map_storage.tiles;
        let tile_events = &mut self.tile_events;
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
            let Some(label) = tiled_map.tilesets_label_by_index.get(&tileset_id) else {
                return;
            };

            let texture_index = match tilemap_texture {
                TilemapTexture::Single(_) => layer_tile.id(),
                #[cfg(not(feature = "atlas"))]
                TilemapTexture::Vector(_) => *tiled_map
                    .tilesets
                    .get(label)
                    .and_then(|t| t.tile_image_offsets.get(&layer_tile.id()))
                    .expect(
                        "The offset into the image vector for tilemap should have been saved during the initial load.",
                    ),
                #[cfg(not(feature = "atlas"))]
                _ => unreachable!(),
            };

            let tile_id = layer_tile.id();
            let tile_entity = commands
                .spawn((
                    Name::new(format!("TiledMapTile({},{})", tile_pos.x, tile_pos.y)),
                    TiledTile,
                    TiledTilesetId(tileset_id),
                    TiledTileId(tile_id),
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(texture_index),
                        flip: TileFlip {
                            x: layer_tile_data.flip_h,
                            y: layer_tile_data.flip_v,
                            d: layer_tile_data.flip_d,
                        },
                        ..default()
                    },
                    ChildOf(tilemap_entity),
                ))
                .id();

            // Handle animated tiles
            if let Some(animated_tile) = get_animated_tile(&tile) {
                commands.entity(tile_entity).insert(animated_tile);
            }

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
        &mut self,
        layer_event: &TiledEvent<LayerCreated>,
        object_layer: tiled::ObjectLayer,
    ) {
        let tiled_map = self.tiled_map;
        let anchor = self.anchor;
        let commands = &mut *self.commands;
        let entity_map = &mut self.map_storage.objects;
        let object_events = &mut self.object_events;
        for (index, object_data) in object_layer.objects().enumerate() {
            let tiled_object = TiledObject::from_object_data(&object_data);
            let mut pos = tiled_map.object_relative_position(&object_data, anchor);

            // For isometric maps, we need to adjust the position of tile objects
            // to match the isometric grid.
            if matches!(
                tilemap_type_from_map(&tiled_map.map),
                TilemapType::Isometric(..)
            ) && let TiledObject::Tile { width, height: _ } = tiled_object
            {
                pos.x -= width / 2.;
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
                TiledObject::Text { .. } => "Text",
                TiledObject::Rectangle { .. } => "Rectangle",
                TiledObject::Ellipse { .. } => "Ellipse",
                TiledObject::Polygon { .. } => "Polygon",
                TiledObject::Polyline { .. } => "Polyline",
                TiledObject::Capsule { .. } => "Capsule",
            };

            let object_entity = commands
                .spawn((
                    Name::new(format!("{object_kind}({})", object_data.name)),
                    ChildOf(layer_event.origin),
                    TiledMapReference(layer_event.map_entity().unwrap()),
                    TiledObjectId(object_data.id()),
                    tiled_object.clone(),
                    TiledName(object_data.name.clone()),
                    transform,
                    match &object_data.visible {
                        true => Visibility::Inherited,
                        false => Visibility::Hidden,
                    },
                ))
                .id();

            #[cfg(feature = "text")]
            if let TiledObject::Text {
                width,
                height,
                text,
                offset,
                font_size,
            } = tiled_object
            {
                commands.spawn((
                    ChildOf(object_entity),
                    Name::new("TiledObjectTextBox"),
                    TiledObjectTextBox,
                    Aabb::from_min_max(Vec3::ZERO, Vec3::new(width, height, 0.)),
                    Transform::from_translation(Vec3::ZERO),
                    Anchor::TOP_LEFT,
                    Visibility::Inherited,
                    children![(
                        Name::new("TiledObjectText"),
                        TiledObjectText,
                        Text2d::new(text),
                        TextFont {
                            font_size: FontSize::Px(font_size as f32),
                            ..default()
                        },
                        Anchor::from(offset),
                        Transform::from_translation(Vec3::new(
                            offset.x * width + width / 2.,
                            offset.y * height - height / 2.,
                            1.
                        )),
                        Visibility::Inherited,
                    )],
                ));
            }

            // Handle objects containing tile data:
            // we want to add a Sprite component to the object entity
            // and possibly an animation component if the tile is animated.
            match handle_tile_object(&object_data, tiled_map) {
                (Some((sprite, offset_transform)), None) => {
                    commands.spawn((
                        Name::new("TiledObjectVisual"),
                        TiledObjectVisualOf(object_entity),
                        ChildOf(object_entity),
                        sprite,
                        Anchor::BOTTOM_LEFT,
                        offset_transform,
                    ));
                }
                (Some((sprite, offset_transform)), Some(animation)) => {
                    commands.spawn((
                        Name::new("TiledObjectVisual"),
                        TiledObjectVisualOf(object_entity),
                        ChildOf(object_entity),
                        sprite,
                        Anchor::BOTTOM_LEFT,
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

    fn spawn_image_layer(
        &mut self,
        layer_event: &TiledEvent<LayerCreated>,
        image_layer: tiled::ImageLayer,
    ) {
        let tiled_map = self.tiled_map;
        let anchor = self.anchor;
        if let Some(image) = &image_layer.image {
            let Some(handle) = layer_event
                .layer_id()
                .and_then(|id| tiled_map.images.get(&id))
            else {
                return;
            };

            let image_size = Vec2::new(image.width as f32, image.height as f32);
            let image_position = tiled_map.screen_origin(anchor);
            self.commands.spawn((
                Name::new(format!("Image({})", image.source.display())),
                TiledImage {
                    base_position: image_position,
                    base_size: image_size,
                },
                ChildOf(layer_event.origin),
                TiledMapReference(layer_event.map_entity().unwrap()),
                Sprite {
                    image: handle.clone(),
                    custom_size: Some(image_size),
                    image_mode: SpriteImageMode::Tiled {
                        tile_x: image_layer.repeat_x,
                        tile_y: image_layer.repeat_y,
                        stretch_value: 1.,
                    },
                    ..default()
                },
                Anchor::TOP_LEFT,
                Transform::from_translation(image_position.extend(0.)),
            ));
        }
    }
}

fn handle_tile_object(
    object: &tiled::Object,
    tiled_map: &TiledMapAsset,
) -> (Option<(Sprite, Transform)>, Option<TiledAnimation>) {
    let Some(tile) = (*object).get_tile() else {
        return (None, None);
    };

    // Assume tile objets always have a rectangular shape
    let tiled::ObjectShape::Rect { width, height } = object.shape else {
        return (None, None);
    };

    let label = match tile.tileset_location() {
        tiled::TilesetLocation::Map(tileset_index) => {
            let tileset_index = *tileset_index as u32;
            let Some(label) = tiled_map.tilesets_label_by_index.get(&tileset_index) else {
                log::warn!("Skipped object with missing tileset (index: {tileset_index})");
                return (None, None);
            };
            label
        }
        tiled::TilesetLocation::Template(tileset) => {
            let Some(label) = tileset_label(tileset) else {
                log::warn!("Skipped object with missing template");
                return (None, None);
            };
            &label.to_owned()
        }
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

    let Some(sprite) = tiled_map.tilesets.get(label).and_then(|t| {
        match &t.tilemap_texture {
            TilemapTexture::Single(single) => {
                t.texture_atlas_layout_handle.as_ref().map(|handle| {
                    Sprite {
                        image: single.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: handle.clone(),
                            index: tile.id() as usize,
                        }),
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
    let animation = tile.get_tile().and_then(|t| {
        t.animation.as_ref().map(|frames| {
            let frame_vec: Vec<(usize, f32)> = frames
                .iter()
                .map(|f| (f.tile_id as usize, f.duration as f32 / 1000.0))
                .collect();
            let first_duration = frame_vec.first().map(|(_, d)| *d).unwrap_or(0.1);
            TiledAnimation {
                frames: frame_vec,
                current_frame: 0,
                timer: Timer::from_seconds(first_duration, TimerMode::Once),
            }
        })
    });

    (Some((sprite, transform)), animation)
}

// Tile-layer animations go through bevy_ecs_tilemap's `AnimatedTile`, which requires
// consecutive frame IDs and a uniform duration. Object-layer animations use `TiledAnimation`
// instead, which stores per-frame (index, duration) pairs and has no such constraints.
fn get_animated_tile(tile: &tiled::Tile) -> Option<AnimatedTile> {
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
        if let Some(id) = previous_tile_id
            && frame.tile_id != id + 1
        {
            log::warn!("Animated tile with non-aligned frame tiles is currently not supported");
            return None;
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
