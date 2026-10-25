//! Systems and utilities for spawning Tiled map entities.
//!
//! This module contains logic for instantiating Bevy entities and components from Tiled map data.
//! It handles the creation of map layers, tiles, objects, and their associated components in the ECS world,
//! enabling the rendering and interaction of Tiled maps within a Bevy application.
//!
//! Only Tiled data components are spawned here: rendering components (tilemaps, sprites, texts)
//! and physics colliders are derived from these data later on, so they are also updated
//! when these data are modified after the map has been spawned.

use crate::{prelude::*, tiled::event::TiledMessageWriters};
use bevy::prelude::*;

#[cfg(feature = "user_properties")]
use crate::tiled::properties::command::PropertiesCommandExt;

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_map(
    commands: &mut Commands,
    map_entity: Entity,
    map_asset_id: AssetId<TiledMapAsset>,
    tiled_map: &TiledMapAsset,
    map_storage: &mut TiledMapStorage,
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
        anchor,
        layer_z_offset: layer_z_offset.0,
        // Order of the differents layers in the .TMX file is important:
        // a layer appearing last in the .TMX should appear above previous layers
        // Start with a negative offset so in the end we end up with the top layer at Z-offset from settings
        acc_z_offset: tiled_map.map.layers().len() as f32 * (-layer_z_offset.0),
        layer_events: Vec::new(),
        object_events: Vec::new(),
        tile_events: Vec::new(),
    };
    ctx.spawn_layers(tiled_map.map.layers(), map_entity, &map_event);
    #[cfg(feature = "user_properties")]
    ctx.insert_properties();

    let SpawnContext {
        commands,
        layer_events,
        object_events,
        tile_events,
        ..
    } = ctx;

    // Send events and trigger observers, only once all the map entities have been spawned
    map_event.send(commands, &mut message_writers.map_created);

    for e in layer_events {
        e.send(commands, &mut message_writers.layer_created);
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
    anchor: &'a TilemapAnchor,
    /// Z offset between two consecutive layers
    layer_z_offset: f32,
    /// Z offset of the last spawned layer
    acc_z_offset: f32,
    /// Events sent once the whole map has been spawned
    layer_events: Vec<TiledEvent<LayerCreated>>,
    object_events: Vec<TiledEvent<ObjectCreated>>,
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
                    self.spawn_tiles_layer(&layer_event, tile_layer);
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
        tiles_layer: tiled::TileLayer,
    ) {
        let tiled_map = self.tiled_map;
        let map_entity = self.map_entity;
        let commands = &mut *self.commands;
        let map_storage = &mut *self.map_storage;
        let tile_events = &mut self.tile_events;
        let layer_entity = layer_event.origin;

        // Only spawn tiles data: associated tilemaps, which hold the actual rendering
        // components, are derived from these data in `TiledPostUpdateSystems::SyncRender`
        tiled_map.for_each_tile(&tiles_layer, |layer_tile, layer_tile_data, tile_pos, _| {
            let tileset_id = layer_tile.tileset_index() as u32;
            let tile_id = layer_tile.id();
            let tile_entity = commands
                .spawn((
                    Name::new(format!("TiledMapTile({},{})", tile_pos.x, tile_pos.y)),
                    TiledTile,
                    TiledTilesetId(tileset_id),
                    TiledTileId(tile_id),
                    tile_pos,
                    TileFlip {
                        x: layer_tile_data.flip_h,
                        y: layer_tile_data.flip_v,
                        d: layer_tile_data.flip_d,
                    },
                    ChildOf(layer_entity),
                    TiledMapReference(map_entity),
                ))
                .id();

            // Handle custom tiles (with user properties)
            if layer_tile
                .get_tile()
                .is_some_and(|tile| !tile.properties.is_empty())
            {
                let tile_event = layer_event
                    .transmute(Some(tile_entity), TileCreated)
                    .with_tile(tile_entity, tile_pos, tileset_id, tile_id)
                    .to_owned();
                tile_events.push(tile_event);
            }

            // Update map storage with tile entity
            map_storage.insert_tile((tileset_id, tile_id), tile_entity);
        });
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
            let tiled_object = TiledObject::from_object_data(tiled_map, &object_data);
            let mut pos = tiled_map.object_relative_position(&object_data, anchor);

            // For isometric maps, we need to adjust the position of tile objects
            // to match the isometric grid.
            if matches!(
                tilemap_type_from_map(&tiled_map.map),
                TilemapType::Isometric(..)
            ) && let TiledObject::Tile { width, .. } = tiled_object
            {
                pos.x -= width / 2.;
            }

            let transform = Transform::from_isometry(
                Isometry3d::from_translation(pos.extend(index as f32 * 0.001))
                    * Isometry3d::from_rotation(Quat::from_rotation_z(f32::to_radians(
                        -object_data.rotation,
                    ))),
            );

            // Only spawn object data: its visual representation (if any) is derived
            // from the TiledObject component in `TiledPostUpdateSystems::SyncRender`
            let object_entity = commands
                .spawn((
                    Name::new(format!("{}({})", tiled_object.kind(), object_data.name)),
                    ChildOf(layer_event.origin),
                    TiledMapReference(self.map_entity),
                    TiledObjectId(object_data.id()),
                    tiled_object,
                    TiledName(object_data.name.clone()),
                    transform,
                    match &object_data.visible {
                        true => Visibility::Inherited,
                        false => Visibility::Hidden,
                    },
                ))
                .id();

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
        let Some(image) = &image_layer.image else {
            return;
        };
        let Some(handle) = layer_event
            .layer_id()
            .and_then(|id| tiled_map.images.get(&id))
        else {
            return;
        };

        // Only spawn image data: the associated Sprite is derived from
        // the TiledImage component in `TiledPostUpdateSystems::SyncRender`
        self.commands.spawn((
            Name::new(format!("Image({})", image.source.display())),
            TiledImage {
                image: handle.clone(),
                base_position: tiled_map.screen_origin(self.anchor),
                base_size: Vec2::new(image.width as f32, image.height as f32),
                repeat_x: image_layer.repeat_x,
                repeat_y: image_layer.repeat_y,
            },
            ChildOf(layer_event.origin),
            TiledMapReference(self.map_entity),
        ));
    }
}
