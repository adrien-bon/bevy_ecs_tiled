use bevy::{
    app::{HierarchyPropagatePlugin, Propagate},
    asset::RenderAssetUsages,
    camera::{RenderTarget, visibility::RenderLayers},
    color::palettes::tailwind::CYAN_100,
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat, TextureUsages},
};
use bevy_ecs_tiled::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_minimap);
    app.add_plugins(HierarchyPropagatePlugin::<
        RenderLayers,
        (Without<TiledImage>, Without<HideOnMinimap>),
    >::new(Update));
    app.add_observer(
        |map_created: On<TiledEvent<MapCreated>>, mut commands: Commands| {
            commands
                .entity(map_created.origin)
                .insert(Propagate(RenderLayers::from_layers(&[
                    DEFAULT_RENDER_LAYER,
                    MINIMAP_RENDER_LAYER,
                ])));
        },
    );
}

pub const DEFAULT_RENDER_LAYER: usize = 0;
pub const MINIMAP_RENDER_LAYER: usize = 1;

/// Size of the minimap, in logical pixels.
const MINIMAP_SIZE: Vec2 = Vec2::new(300., 150.);

#[derive(Component)]
pub struct HideOnMinimap;

/// A colored rectangle only rendered on the minimap, to be spawned as a child of the entity to track.
pub fn minimap_marker(color: impl Into<Color>) -> impl Bundle {
    (
        Name::new("Minimap Marker"),
        Sprite {
            custom_size: Some(Vec2::new(32., 96.)),
            color: color.into(),
            ..default()
        },
        Transform::from_xyz(0., 0., 100.0),
        // Render on minimap only, position is inherited from parent
        RenderLayers::layer(MINIMAP_RENDER_LAYER),
    )
}

fn setup_minimap(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_uninit(
        default(),
        TextureDimension::D2,
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::all(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let image_handle = images.add(image);

    let camera = commands
        .spawn((
            Name::new("Minimap Camera"),
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scale: 16.,
                ..OrthographicProjection::default_2d()
            }),
            Camera {
                order: 1, // After main camera at default order 0
                // Minimap blue-ish background
                clear_color: ClearColorConfig::Custom(Color::Srgba(CYAN_100).with_alpha(0.6)),
                ..default()
            },
            RenderTarget::Image(image_handle.clone().into()),
            RenderLayers::layer(MINIMAP_RENDER_LAYER),
        ))
        .id();

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Px(MINIMAP_SIZE.x),
            height: Val::Px(MINIMAP_SIZE.y),
            ..default()
        },
        ViewportNode::new(camera),
    ));
}
