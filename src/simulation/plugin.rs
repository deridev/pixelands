use std::time::Duration;

use bevy::{
    image::ImageSampler,
    input::mouse::MouseWheel,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::Anchor,
    time::common_conditions::on_timer,
};
use bevy_egui::EguiContexts;

use crate::{
    common::math,
    constants::{CHUNK_SIZE, RESOLUTION},
    coordinates::{chunk_to_world_position, world_to_chunk_position, world_to_element_position},
};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Component)]
pub struct WorldChunk {
    pub position: IVec2,
}

#[derive(Component)]
pub struct MainCameraState {
    pub acceleration: f32,
    pub friction: f32,
    pub max_speed: f32,
    pub velocity: Vec2,
}

#[derive(Resource)]
pub struct Resolution(pub f32);

#[derive(Resource)]
pub struct LastMousePosition(pub IVec2);

#[derive(Resource)]
pub struct SelectedElement(pub ElementKind);

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_simulation)
            .add_systems(
                Update,
                tick_simulation.run_if(on_timer(Duration::from_millis(30))),
            )
            .add_systems(PreUpdate, create_fresh_chunks)
            .add_systems(Update, (draw, render_simulation).chain())
            .add_systems(
                Update,
                (
                    zoom_camera,
                    toggle_active,
                    walk_camera,
                    change_selected_element,
                ),
            )
            .add_systems(PostUpdate, update_last_mouse_position)
            .insert_resource(Resolution(RESOLUTION as f32))
            .insert_resource(LastMousePosition(IVec2::ZERO))
            .insert_resource(SelectedElement(ElementKind::Sand));
    }
}

pub fn setup_simulation(mut commands: Commands) {
    commands
        .spawn(Camera2d)
        .insert(MainCameraState {
            acceleration: 600.0,
            friction: 0.1,
            max_speed: 1000.0,
            velocity: Vec2::ZERO,
        })
        .insert(Transform::from_xyz(
            (CHUNK_SIZE as f32 * RESOLUTION as f32) / 2.0,
            (CHUNK_SIZE as f32 * RESOLUTION as f32) / 2.0,
            100.0,
        ));

    commands.insert_resource(Sandbox::new());
}

pub fn zoom_camera(
    mut resolution: ResMut<Resolution>,
    mut scroll_event: EventReader<MouseWheel>,
    mut q_chunks: Query<(&WorldChunk, &mut Transform, &mut Sprite)>,
) {
    use bevy::input::mouse::MouseScrollUnit;

    let mut delta = 0.0;
    for ev in scroll_event.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                delta += ev.y as f32 / 6.0;
            }
            MouseScrollUnit::Pixel => {}
        }
    }

    resolution.0 = (resolution.0 + delta).clamp(0.3, 20.0);

    for (WorldChunk { position }, mut transform, mut sprite) in q_chunks.iter_mut() {
        sprite.custom_size = Some(Vec2::splat(resolution.0 * CHUNK_SIZE as f32));
        transform.translation = chunk_to_world_position(*position).extend(1.0) * resolution.0;
    }
}

pub fn toggle_active(mut sandbox: ResMut<Sandbox>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        sandbox.active = !sandbox.active;
    }
}

pub fn walk_camera(
    mut camera: Query<(&mut MainCameraState, &mut Transform)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((mut camera_state, mut transform)) = camera.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let acceleration = camera_state.acceleration;
    let direction = direction.normalize_or_zero();
    let friction = camera_state.friction;
    if direction == Vec2::ZERO {
        camera_state.velocity = camera_state.velocity.lerp(Vec2::ZERO, friction);
    } else {
        camera_state.velocity += direction * acceleration;
        camera_state.velocity = camera_state
            .velocity
            .clamp_length_max(camera_state.max_speed);
    }

    transform.translation += (camera_state.velocity * time.delta_secs()).extend(0.0);
}

pub fn create_fresh_chunks(
    mut sandbox: ResMut<Sandbox>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    resolution: Res<Resolution>,
) {
    for fresh_chunk_position in sandbox.fresh_chunks.drain(..) {
        let mut image = Image::new_fill(
            Extent3d {
                height: CHUNK_SIZE as u32,
                width: CHUNK_SIZE as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
            Default::default(),
        );
        image.sampler = ImageSampler::nearest();

        let handle = images.add(image);

        commands
            .spawn(WorldChunk {
                position: fresh_chunk_position,
            })
            .insert(Sprite {
                image: handle,
                anchor: Anchor::TopLeft,
                custom_size: Some(Vec2::splat(resolution.0 * CHUNK_SIZE as f32)),
                ..Default::default()
            })
            .insert(Transform::from_translation(
                chunk_to_world_position(fresh_chunk_position).extend(1.0) * resolution.0,
            ));
    }
}

pub fn change_selected_element(
    mut selected_element: ResMut<SelectedElement>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        selected_element.0 = ElementKind::Sand;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        selected_element.0 = ElementKind::Stone;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        selected_element.0 = ElementKind::Water;
    }
}

pub fn update_last_mouse_position(
    mut last_mouse_position: ResMut<LastMousePosition>,
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(window) = q_window.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(mouse_position) = camera
        .viewport_to_world(camera_transform, cursor_position)
        .map(|p| p.origin.truncate())
    else {
        return;
    };

    last_mouse_position.0 = mouse_position.as_ivec2();
}

pub fn draw(
    mut sandbox: ResMut<Sandbox>,
    mut egui_ctx: EguiContexts,
    last_mouse_position: Res<LastMousePosition>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    resolution: Res<Resolution>,
    selected_element: Res<SelectedElement>,
) {
    if egui_ctx.ctx_mut().wants_pointer_input() {
        return;
    }

    let is_deleting = mouse_input.pressed(MouseButton::Right);
    if !mouse_input.pressed(MouseButton::Left) && !is_deleting {
        return;
    }

    let Ok(window) = q_window.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(mouse_position) = camera
        .viewport_to_world(camera_transform, cursor_position)
        .map(|p| p.origin.truncate())
    else {
        return;
    };

    let iter = math::GridLineIterator::new(last_mouse_position.0, mouse_position.as_ivec2());
    for pos in iter {
        let mut pos = pos.as_vec2();
        pos /= resolution.0;
        pos.y *= -1.0;

        let chunk_position = world_to_chunk_position(pos);

        let chunk = match sandbox.get_shared_chunk(chunk_position) {
            Some(chunk) => chunk,
            None => {
                let shared = sandbox.add_chunk(Chunk::new(chunk_position));
                shared
            }
        };

        let local_position = world_to_element_position(pos);

        if !chunk.read().is_empty(local_position) && !is_deleting {
            continue;
        }

        let element_kind = if is_deleting {
            ElementKind::Air
        } else {
            selected_element.0
        };

        chunk.write().set_element(
            local_position,
            Element {
                wframe: 0,
                color: element_kind.base_color(),
                velocity: Vec2::ZERO,
                kind: element_kind,
            },
        );
    }
}

pub fn tick_simulation(mut sandbox: ResMut<Sandbox>) {
    sandbox.tick();
}

pub fn render_simulation(
    sandbox: ResMut<Sandbox>,
    mut chunks: Query<(&mut WorldChunk, &Sprite)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (chunk, sprite) in chunks.iter_mut() {
        let image = images.get_mut(&sprite.image).unwrap();
        let chunk = sandbox.get_chunk(chunk.position).unwrap();
        if !chunk.active() {
            continue;
        }

        let data = image.data.as_mut().unwrap();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let element = chunk.get_element(IVec2::new(x as i32, y as i32));
                let color = element.color;

                let index = (y * CHUNK_SIZE + x) * 4;
                data[index] = color.0;
                data[index + 1] = color.1;
                data[index + 2] = color.2;
                data[index + 3] = 255;
            }
        }
    }
}
