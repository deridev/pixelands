mod common;
mod constants;
mod coordinates;
mod debug_ui;
mod simulation;

use bevy::{
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PresentMode,
};
use debug_ui::DebugUiPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Sandbox".to_string(),
                        present_mode: PresentMode::Immediate,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(DebugUiPlugin)
        .add_plugins(simulation::plugin::SimulationPlugin)
        .add_systems(Startup, spawn_pivot)
        .add_systems(PostUpdate, move_piv_towards_mouse)
        .run();
}

#[derive(Component)]
struct Piv;

fn spawn_pivot(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            height: 2,
            width: 2,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[225, 255, 255, 100],
        TextureFormat::Rgba8UnormSrgb,
        Default::default(),
    );
    image.sampler = ImageSampler::nearest();

    let handle = images.add(image);

    commands
        .spawn(Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)))
        .insert(Sprite {
            image: handle,
            custom_size: Some(Vec2::splat(4.0)),
            ..Default::default()
        })
        .insert(Piv);
}

fn move_piv_towards_mouse(
    mut piv: Query<&mut Transform, With<Piv>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_window: Query<&Window>,
) {
    let Ok(window) = q_window.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };

    let Ok(mouse_position) = camera
        .viewport_to_world(camera_transform, cursor_position)
        .map(|p| p.origin.truncate())
    else {
        return;
    };

    let Ok(mut piv) = piv.single_mut() else {
        return;
    };

    piv.translation.x = mouse_position.x;
    piv.translation.y = mouse_position.y;
}
