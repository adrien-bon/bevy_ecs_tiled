//! ECS components for Tiled objects.
//!
//! This module defines Bevy components used to represent Tiled objects within the ECS world.

use crate::prelude::{geo::Centroid, *};
use crate::tiled::{helpers::iso_projection, map::loader::tileset_label};
use ::tiled::{HorizontalAlignment, VerticalAlignment};
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

/// Tiled object ID associated with this [`TiledObject`].
///
/// You can retrieve the corresponding [`tiled::Object`] using the [`object_from_map`] helper function.
#[derive(Component, Default, Reflect, Copy, Clone, Debug, Deref)]
#[reflect(Component, Default, Debug)]
pub struct TiledObjectId(pub u32);

/// Marker [`Component`] for a [`TiledObject`]'s text box.
#[derive(Component, Reflect, Copy, Clone, Debug)]
pub struct TiledObjectTextBox;

/// Marker [`Component`] for a [`TiledObjectTextBox`]'s inner text.
#[derive(Component, Reflect, Copy, Clone, Debug)]
pub struct TiledObjectText;

/// Marker [`Component`] for a Tiled map object.
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform, TiledName, TiledObjectId)]
pub enum TiledObject {
    /// A point shape.
    #[default]
    Point,
    /// A rectangle shape.
    ///
    /// Anchor is at the top-left corner of the rect.
    Rectangle {
        /// The width of the rectangle.
        width: f32,
        /// The height of the rectangle.
        height: f32,
    },
    /// An ellipse shape.
    ///
    /// Anchor is at the top-left corner of the ellipse.
    Ellipse {
        /// The width of the ellipse.
        width: f32,
        /// The height of the ellipse.
        height: f32,
    },
    /// A polygon shape.
    Polygon {
        /// The vertices of the polygon, relative to object center.
        vertices: Vec<Vec2>,
    },
    /// A polyline shape.
    Polyline {
        /// The vertices of the polyline, relative to object center.
        vertices: Vec<Vec2>,
    },
    /// A tile object, which is a reference to a tile in a tileset.
    ///
    /// Anchor is at the bottom-left corner of the tile.
    /// These objects have a child [`TiledObjectVisualOf`] entity holding
    /// their visual representation, which is usually a [`Sprite`].
    Tile {
        /// The width of the tile.
        width: f32,
        /// The height of the tile.
        height: f32,
        /// The tileset ID of the tile.
        ///
        /// Use [`TiledMapAsset::tileset`] to retrieve the corresponding [`tiled::Tileset`], since it
        /// can be a tileset only referenced by a template.
        tileset_id: u32,
        /// The [`tiled::TileId`] of the tile.
        tile_id: tiled::TileId,
        /// Whether the tile is flipped horizontally.
        flip_x: bool,
        /// Whether the tile is flipped vertically.
        flip_y: bool,
    },
    /// A text object, which contains text data.
    Text {
        /// the width of the text box.
        width: f32,
        /// the height of the text box.
        height: f32,
        /// the text content.
        text: String,
        /// the vertical and horizontal alignment of the text.
        offset: Vec2,
        /// the font size in pixels.
        font_size: usize,
    },
    /// A capsule shape.
    Capsule {
        /// The width of the capsule.
        width: f32,
        /// The height of the capsule.
        height: f32,
    },
}

impl TiledObject {
    const ELLIPSE_NUM_POINTS: u32 = 20;

    fn capsule_vertices(width: f32, height: f32) -> Vec<Vec2> {
        let num_points = Self::ELLIPSE_NUM_POINTS / 2;

        if width >= height {
            let radius = height / 2.0;
            let center_y = -height / 2.0;
            let left_center = Vec2::new(radius, center_y);
            let right_center = Vec2::new(width - radius, center_y);
            let mut points = Vec::with_capacity((num_points * 2) as usize);

            for i in 0..=num_points {
                let theta = -std::f32::consts::PI / 2.0
                    + std::f32::consts::PI * (i as f32 / num_points as f32);
                let x = left_center.x - radius * theta.cos();
                let y = left_center.y + radius * theta.sin();
                points.push(Vec2::new(x, y));
            }

            for i in 0..=num_points {
                let theta = std::f32::consts::PI / 2.0
                    - std::f32::consts::PI * (i as f32 / num_points as f32);
                let x = right_center.x + radius * theta.cos();
                let y = right_center.y + radius * theta.sin();
                points.push(Vec2::new(x, y));
            }

            points
        } else {
            let radius = width / 2.0;
            let center_x = width / 2.0;
            let top_center = Vec2::new(center_x, -radius);
            let bottom_center = Vec2::new(center_x, -(height - radius));
            let mut points = Vec::with_capacity((num_points * 2) as usize);

            for i in 0..=num_points {
                let theta =
                    std::f32::consts::PI - std::f32::consts::PI * (i as f32 / num_points as f32);
                let x = top_center.x + radius * theta.cos();
                let y = top_center.y + radius * theta.sin();
                points.push(Vec2::new(x, y));
            }

            for i in 0..=num_points {
                let theta = std::f32::consts::PI * (i as f32 / num_points as f32);
                let x = bottom_center.x + radius * theta.cos();
                let y = bottom_center.y - radius * theta.sin();
                points.push(Vec2::new(x, y));
            }

            points
        }
    }

    /// Returns the kind of this [`TiledObject`] as a string.
    pub fn kind(&self) -> &'static str {
        match self {
            TiledObject::Point => "Point",
            TiledObject::Tile { .. } => "Tile",
            TiledObject::Text { .. } => "Text",
            TiledObject::Rectangle { .. } => "Rectangle",
            TiledObject::Ellipse { .. } => "Ellipse",
            TiledObject::Polygon { .. } => "Polygon",
            TiledObject::Polyline { .. } => "Polyline",
            TiledObject::Capsule { .. } => "Capsule",
        }
    }

    /// Creates a new [`TiledObject`] from the provided [`tiled::ObjectData`].
    ///
    /// The [`TiledMapAsset`] is needed to resolve the tileset ID of tile objects.
    pub fn from_object_data(map_asset: &TiledMapAsset, object_data: &tiled::ObjectData) -> Self {
        if let Some(tile_data) = object_data.tile_data() {
            if let tiled::ObjectShape::Rect { width, height } = object_data.shape {
                let tileset_id = match tile_data.tileset_location() {
                    tiled::TilesetLocation::Map(index) => Some(*index as u32),
                    tiled::TilesetLocation::Template(tileset) => tileset_label(tileset)
                        .and_then(|label| map_asset.tilesets_id_by_label.get(&label).copied()),
                };
                let Some(tileset_id) = tileset_id else {
                    warn!(
                        "Cannot find tileset for tile object '{}', it will be spawned as a point",
                        object_data.name
                    );
                    return TiledObject::default();
                };
                TiledObject::Tile {
                    width,
                    height,
                    tileset_id,
                    tile_id: tile_data.id(),
                    flip_x: tile_data.flip_h,
                    flip_y: tile_data.flip_v,
                }
            } else {
                warn!(
                    "Object with tile data should have a rectangle shape, but found {:?}",
                    object_data.shape
                );
                TiledObject::default()
            }
        } else {
            match object_data.shape.clone() {
                tiled::ObjectShape::Point { .. } => TiledObject::Point,
                tiled::ObjectShape::Rect { width, height } => {
                    TiledObject::Rectangle { width, height }
                }
                tiled::ObjectShape::Ellipse { width, height } => {
                    TiledObject::Ellipse { width, height }
                }
                tiled::ObjectShape::Polygon { points } => TiledObject::Polygon {
                    vertices: points.into_iter().map(|(x, y)| Vec2::new(x, -y)).collect(),
                },
                tiled::ObjectShape::Polyline { points } => TiledObject::Polyline {
                    vertices: points.into_iter().map(|(x, y)| Vec2::new(x, -y)).collect(),
                },
                tiled::ObjectShape::Text {
                    width,
                    height,
                    text,
                    valign,
                    halign,
                    pixel_size,
                    ..
                } => {
                    let offset = match (valign, halign) {
                        (VerticalAlignment::Top, HorizontalAlignment::Left)
                        | (VerticalAlignment::Top, HorizontalAlignment::Justify) => {
                            Vec2::new(-0.5, 0.5)
                        }
                        (VerticalAlignment::Bottom, HorizontalAlignment::Left)
                        | (VerticalAlignment::Bottom, HorizontalAlignment::Justify) => {
                            Vec2::new(-0.5, -0.5)
                        }
                        (VerticalAlignment::Center, HorizontalAlignment::Left)
                        | (VerticalAlignment::Center, HorizontalAlignment::Justify) => {
                            Vec2::new(-0.5, 0.0)
                        }
                        (VerticalAlignment::Top, HorizontalAlignment::Right) => Vec2::new(0.5, 0.5),
                        (VerticalAlignment::Bottom, HorizontalAlignment::Right) => {
                            Vec2::new(0.5, -0.5)
                        }
                        (VerticalAlignment::Center, HorizontalAlignment::Right) => {
                            Vec2::new(0.5, 0.0)
                        }
                        (VerticalAlignment::Top, HorizontalAlignment::Center) => {
                            Vec2::new(0.0, 0.5)
                        }
                        (VerticalAlignment::Bottom, HorizontalAlignment::Center) => {
                            Vec2::new(0.0, -0.5)
                        }
                        (VerticalAlignment::Center, HorizontalAlignment::Center) => Vec2::ZERO,
                    };

                    TiledObject::Text {
                        width,
                        height,
                        text,
                        offset,
                        font_size: pixel_size,
                    }
                }
                tiled::ObjectShape::Capsule { width, height } => {
                    TiledObject::Capsule { width, height }
                }
            }
        }
    }

    /// Apply rotation and scaling to given coordinate.
    fn apply_rotation_and_scaling(
        inverse_rotation: bool,
        vertex: Vec2,
        transform: &GlobalTransform,
    ) -> Vec2 {
        let mut rotation = transform.rotation().to_euler(EulerRot::ZYX).0;
        if inverse_rotation {
            rotation *= -1.;
        }
        let (cos, sin) = (rotation.cos(), rotation.sin());
        Vec2 {
            x: (vertex.x * cos - vertex.y * sin) * transform.scale().x,
            y: (vertex.x * sin + vertex.y * cos) * transform.scale().y,
        }
    }

    /// Returns the center position of the object in world space.
    ///
    /// The center is computed from the object's vertices, taking into account its shape and transformation.
    ///
    /// # Arguments
    /// * `transform` - The global transform to apply to the object.
    /// * `isometric_projection` - Wheter or not to perform an isometric projection.
    /// * `tilemap_size` - Size of the tilemap in tiles.
    /// * `grid_size` - Size of each tile on the grid in pixels.
    /// * `offset` - Global map offset to apply.
    ///
    /// # Returns
    /// * `Option<geo::Coord<f32>>` - The computed center, or `None` if not applicable.
    pub fn center(
        &self,
        transform: &GlobalTransform,
        isometric_projection: bool,
        tilemap_size: &TilemapSize,
        grid_size: &TilemapGridSize,
        offset: Vec2,
    ) -> Option<geo::Coord<f32>> {
        geo::MultiPoint::from(self.vertices(
            transform,
            isometric_projection,
            tilemap_size,
            grid_size,
            offset,
        ))
        .centroid()
        .map(|p| geo::Coord { x: p.x(), y: p.y() })
    }

    /// Returns the vertices of the object in world space.
    ///
    /// Vertices are calculated based on the object's shape and its transformation (translation, rotation, scale).
    /// For isometric maps, the vertices are projected through the isometric transformation.
    ///
    /// # Arguments
    /// * `transform` - The global transform to apply to the object.
    /// * `isometric_projection` - Wheter or not to perform an isometric projection.
    /// * `tilemap_size` - Size of the tilemap in tiles.
    /// * `grid_size` - Size of each tile on the grid in pixels.
    /// * `offset` - Global map offset to apply.
    ///
    /// # Returns
    /// * `Vec<geo::Coord<f32>>` - The transformed vertices.
    pub fn vertices(
        &self,
        transform: &GlobalTransform,
        isometric_projection: bool,
        tilemap_size: &TilemapSize,
        grid_size: &TilemapGridSize,
        offset: Vec2,
    ) -> Vec<geo::Coord<f32>> {
        // Get object world position
        let object_world_pos = geo::Coord {
            x: transform.translation().x,
            y: transform.translation().y,
        };

        // Generate shape vertices relative to origin
        match self {
            TiledObject::Point => vec![Vec2::ZERO],
            TiledObject::Tile { width, height, .. } => {
                vec![
                    Vec2::new(0., 0.),          // Bottom-left relative to object
                    Vec2::new(0., *height),     // Top-left
                    Vec2::new(*width, *height), // Top-right
                    Vec2::new(*width, 0.),      // Bottom-right
                ]
            }
            TiledObject::Rectangle { width, height } | TiledObject::Text { width, height, .. } => {
                vec![
                    Vec2::new(0., 0.),           // Top-left relative to object
                    Vec2::new(*width, 0.),       // Top-right
                    Vec2::new(*width, -*height), // Bottom-right
                    Vec2::new(0., -*height),     // Bottom-left
                ]
            }
            TiledObject::Ellipse { width, height } => (0..Self::ELLIPSE_NUM_POINTS)
                .map(|i| {
                    let theta =
                        2.0 * std::f32::consts::PI * (i as f32) / (Self::ELLIPSE_NUM_POINTS as f32);
                    let local_x = width / 2.0 * theta.cos() + width / 2.0;
                    let local_y = height / 2.0 * theta.sin() - height / 2.0;
                    Vec2::new(local_x, local_y)
                })
                .collect(),
            TiledObject::Capsule { width, height } => Self::capsule_vertices(*width, *height),
            TiledObject::Polyline { vertices } | TiledObject::Polygon { vertices } => {
                vertices.clone()
            }
        }
        .into_iter()
        .map(|v| {
            // Only perform isometric projection if requested by caller and if we do not handle a Tile
            if isometric_projection && !matches!(self, TiledObject::Tile { .. }) {
                let offset_projected = iso_projection(
                    Vec2::new(offset.x + v.x, offset.y - v.y),
                    tilemap_size,
                    grid_size,
                );
                let origin_projected = iso_projection(offset, tilemap_size, grid_size);
                let relative_projected = offset_projected - origin_projected;

                let v = Self::apply_rotation_and_scaling(true, relative_projected, transform);
                geo::Coord {
                    x: object_world_pos.x + v.x,
                    y: object_world_pos.y - v.y,
                }
            } else {
                let v = Self::apply_rotation_and_scaling(false, v, transform);
                geo::Coord {
                    x: v.x + object_world_pos.x,
                    y: v.y + object_world_pos.y,
                }
            }
        })
        .collect()
    }

    /// Creates a [`geo::LineString`] from the object's vertices.
    ///
    /// Returns `None` for point and text objects.
    /// For ellipses, rectangles, tiles, and polygons, returns a closed line string.
    /// For polylines, returns an open line string.
    ///
    /// # Arguments
    /// * `transform` - The global transform to apply to the object.
    /// * `isometric_projection` - Wheter or not to perform an isometric projection.
    /// * `tilemap_size` - Size of the tilemap in tiles.
    /// * `grid_size` - Size of each tile on the grid in pixels.
    /// * `offset` - Global map offset to apply.
    ///
    /// # Returns
    /// * `Option<geo::LineString<f32>>` - The resulting line string, or `None` if not applicable.
    pub fn line_string(
        &self,
        transform: &GlobalTransform,
        isometric_projection: bool,
        tilemap_size: &TilemapSize,
        grid_size: &TilemapGridSize,
        offset: Vec2,
    ) -> Option<geo::LineString<f32>> {
        let coords = self.vertices(
            transform,
            isometric_projection,
            tilemap_size,
            grid_size,
            offset,
        );
        match self {
            TiledObject::Point => None,
            TiledObject::Ellipse { .. }
            | TiledObject::Rectangle { .. }
            | TiledObject::Text { .. }
            | TiledObject::Tile { .. }
            | TiledObject::Polygon { .. }
            | TiledObject::Capsule { .. } => {
                let mut line_string = geo::LineString::from(coords);
                line_string.close();
                Some(line_string)
            }
            TiledObject::Polyline { .. } => Some(geo::LineString::new(coords)),
        }
    }

    /// Creates a [`geo::Polygon`] from the object's vertices.
    ///
    /// Returns `None` for polyline, point, and text objects.
    /// For closed shapes, returns the corresponding polygon.
    ///
    /// # Arguments
    /// * `transform` - The global transform to apply to the object.
    /// * `isometric_projection` - Wheter or not to perform an isometric projection.
    /// * `tilemap_size` - Size of the tilemap in tiles.
    /// * `grid_size` - Size of each tile on the grid in pixels.
    /// * `offset` - Global map offset to apply.
    ///
    /// # Returns
    /// * `Option<geo::Polygon<f32>>` - The resulting polygon, or `None` if not applicable.
    pub fn polygon(
        &self,
        transform: &GlobalTransform,
        isometric_projection: bool,
        tilemap_size: &TilemapSize,
        grid_size: &TilemapGridSize,
        offset: Vec2,
    ) -> Option<geo::Polygon<f32>> {
        self.line_string(
            transform,
            isometric_projection,
            tilemap_size,
            grid_size,
            offset,
        )
        .and_then(|ls| match ls.is_closed() {
            true => Some(geo::Polygon::new(ls, vec![])),
            false => None,
        })
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledObject>();
    app.register_type::<TiledObjectId>();
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
                            font_size: FontSize::Px(*font_size as f32),
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
    tileset_id: u32,
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

    let sprite = match &tileset.tilemap_texture {
        TilemapTexture::Single(single) => {
            tileset
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
                })
        }
        #[cfg(not(feature = "atlas"))]
        TilemapTexture::Vector(vector) => tileset
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
