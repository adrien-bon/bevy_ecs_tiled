//! Visual representation of Tiled objects: tile objects sprites and text objects texts.

use crate::prelude::*;
use bevy::{prelude::*, sprite::Anchor};

/// Relationship and Marker [`Component`] for the visual representation of a [`TiledObject`].
///
/// Added on the child [`Entity`] of a [`TiledObject::Tile`] or of a [`TiledObject::Text`].
/// For a [`TiledObject::Tile`], this entity has an associated [`Sprite`] and eventually a [`TiledAnimation`] component.
/// For a [`TiledObject::Text`], this entity is the [`TiledObjectTextBox`].
///
/// Visual entities are derived from the [`TiledObject`] component: they are automatically despawned
/// and spawned again when this component changes, during [`TiledPostUpdateSystems::SyncRender`].
#[derive(Component, Reflect, Copy, Clone, Debug, Deref)]
#[reflect(Component, Debug)]
#[require(Visibility, Transform)]
#[relationship(relationship_target = TiledObjectVisuals)]
pub struct TiledObjectVisualOf(pub Entity);

/// Relationship target [`Component`] pointing to the child [`TiledObjectVisualOf`]s.
#[derive(Component, Reflect, Debug, Deref)]
#[reflect(Component, Debug)]
#[relationship_target(relationship = TiledObjectVisualOf)]
pub struct TiledObjectVisuals(Vec<Entity>);

/// Marker [`Component`] for a [`TiledObject`]'s text box.
#[derive(Component, Reflect, Copy, Clone, Debug)]
pub struct TiledObjectTextBox;

/// Marker [`Component`] for a [`TiledObjectTextBox`]'s inner text.
#[derive(Component, Reflect, Copy, Clone, Debug)]
pub struct TiledObjectText;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledObjectVisualOf>();
    app.register_type::<TiledObjectVisuals>();
    app.add_systems(
        PostUpdate,
        sync_object_visuals.in_set(TiledPostUpdateSystems::SyncRender),
    );
    app.add_observer(on_object_removed);
}

/// Despawn the visual representation of an object when its [`TiledObject`] component is removed.
fn on_object_removed(
    removed: On<Remove<TiledObject>>,
    mut commands: Commands,
    visuals_query: Query<&TiledObjectVisuals>,
) {
    // Visuals are also children of the object: when the object is despawned, they already are
    for visual in visuals_query
        .get(removed.entity)
        .into_iter()
        .flat_map(|v| v.iter())
    {
        commands.entity(visual).try_despawn();
    }
}

/// Spawn the visual representation of new or modified [`TiledObject`]s, removing the previous one.
fn sync_object_visuals(
    mut commands: Commands,
    objects_query: Query<
        (
            Entity,
            &TiledObject,
            Option<&TiledObjectVisuals>,
            Option<&TiledMapReference>,
        ),
        Changed<TiledObject>,
    >,
    maps_query: Query<&TiledMap>,
    assets: Res<Assets<TiledMapAsset>>,
) {
    for (object_entity, object, visuals, map_ref) in objects_query.iter() {
        // Cleanup previous visuals
        for visual in visuals.into_iter().flat_map(|v| v.iter()) {
            commands.entity(visual).despawn();
        }

        match object {
            TiledObject::Tile {
                width,
                height,
                tileset_id,
                tile_id,
                flip_x,
                flip_y,
            } => {
                let Some(map_asset) = map_ref
                    .and_then(|map_ref| maps_query.get(map_ref.0).ok())
                    .and_then(|tiled_map| assets.get(&tiled_map.0))
                else {
                    warn!(
                        "Cannot spawn visual for tile object {object_entity}: missing or not loaded map"
                    );
                    continue;
                };
                let Some((sprite, transform, animation)) = tile_object_visual(
                    map_asset,
                    *tileset_id,
                    *tile_id,
                    Vec2::new(*width, *height),
                    *flip_x,
                    *flip_y,
                ) else {
                    warn!(
                        "Cannot spawn visual for tile object {object_entity}: tile {tile_id} not found in tileset {tileset_id}"
                    );
                    continue;
                };
                let mut visual = commands.spawn((
                    Name::new("TiledObjectVisual"),
                    TiledObjectVisualOf(object_entity),
                    ChildOf(object_entity),
                    sprite,
                    Anchor::BOTTOM_LEFT,
                    transform,
                ));
                if let Some(animation) = animation {
                    visual.insert(animation);
                }
            }
            #[cfg(feature = "text")]
            TiledObject::Text {
                width,
                height,
                text,
                offset,
                font_size,
            } => {
                use bevy::camera::primitives::Aabb;
                let (width, height, offset) = (*width, *height, *offset);
                commands.spawn((
                    ChildOf(object_entity),
                    TiledObjectVisualOf(object_entity),
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
                            font_size: FontSize::Px(*font_size),
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
            _ => {}
        }
    }
}

/// Build the [`Sprite`], offset [`Transform`] and eventual [`TiledAnimation`] of a [`TiledObject::Tile`].
fn tile_object_visual(
    map_asset: &TiledMapAsset,
    tileset_id: TilesetId,
    tile_id: tiled::TileId,
    size: Vec2,
    flip_x: bool,
    flip_y: bool,
) -> Option<(Sprite, Transform, Option<TiledAnimation>)> {
    let tileset = map_asset.tilesets.get(&tileset_id)?;
    let tile = tileset.tileset.get_tile(tile_id)?;

    let unscaled_tile_size = match &tile.image {
        // tile is in image collection
        Some(image) => Vec2::new(image.width as f32, image.height as f32),
        None => Vec2::new(
            tileset.tileset.tile_width as f32,
            tileset.tileset.tile_height as f32,
        ),
    };
    let scale = size / unscaled_tile_size;
    let transform = Transform::from_xyz(
        tileset.tileset.offset_x as f32 * scale.x,
        -tileset.tileset.offset_y as f32 * scale.y,
        0.0,
    );

    let sprite = match &tileset.textures.tilemap_texture {
        TilemapTexture::Single(single) => tileset
            .textures
            .texture_atlas_layout_handle
            .as_ref()
            .map(|handle| Sprite {
                image: single.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: handle.clone(),
                    index: tile_id as usize,
                }),
                flip_x,
                flip_y,
                custom_size: Some(size),
                ..default()
            }),
        #[cfg(not(feature = "atlas"))]
        TilemapTexture::Vector(vector) => tileset
            .textures
            .tile_image_offsets
            .get(&tile_id)
            .and_then(|index| vector.get(*index as usize))
            .map(|image| Sprite {
                image: image.clone(),
                flip_x,
                flip_y,
                custom_size: Some(size),
                ..default()
            }),
        #[cfg(not(feature = "atlas"))]
        _ => None,
    }?;

    // Handle the case of an animated tile
    let animation = tile.animation.as_ref().map(|frames| {
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
    });

    Some((sprite, transform, animation))
}
