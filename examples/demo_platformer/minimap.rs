use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::render::view::RenderLayers;


pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_minimap);
}

fn setup_minimap(mut commands: Commands, window: Single<&Window>) {
    let mut minimap_projection = OrthographicProjection::default_2d();
    minimap_projection.scale = 16.0;
    let minimap_width = 400;
    let minimap_height = 300;
    commands.spawn((
        Name::new("Minimap Camera"),
        Camera2d,
        Projection::Orthographic(minimap_projection),
        Camera {
            viewport: Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(minimap_width, minimap_height),
                ..default()
            }),
            order: 1, // After main camera at default order 0
            ..default()
        },
        RenderLayers::layer(1),
    ));

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
        BackgroundColor(Color::srgba(0.243, 0.361, 0.522, 0.82)),
    ));
}