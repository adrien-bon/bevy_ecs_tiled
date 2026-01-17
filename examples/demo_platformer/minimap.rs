use bevy::{
    app::{HierarchyPropagatePlugin, Propagate},
    asset::RenderAssetUsages,
    camera::{visibility::RenderLayers, RenderTarget},
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
                .entity(map_created.event().origin)
                .insert(Propagate(RenderLayers::from_layers(&[
                    DEFAULT_RENDER_LAYER,
                    MINIMAP_RENDER_LAYER,
                ])));
        },
    );
}

pub const DEFAULT_RENDER_LAYER: usize = 0;
pub const MINIMAP_RENDER_LAYER: usize = 1;

#[derive(Component)]
pub struct HideOnMinimap;

fn setup_minimap(
    mut commands: Commands,
    window: Single<&Window>,
    mut images: ResMut<Assets<Image>>,
) {
    let minimap_width = 400;
    let minimap_height = 300;

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
                clear_color: ClearColorConfig::Custom(Color::Srgba(CYAN_100).with_alpha(0.6)),
                ..default()
            },
            RenderTarget::Image(image_handle.clone().into()),
            RenderLayers::layer(MINIMAP_RENDER_LAYER),
        ))
        .id();

    // Minimap blue-ish background
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Px(minimap_width as f32 / window.resolution.scale_factor()),
            height: Val::Px(minimap_height as f32 / window.resolution.scale_factor()),
            ..default()
        },
        ViewportNode::new(camera),
    ));
}
