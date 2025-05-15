use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::{
    constants::CHUNK_SIZE,
    simulation::{
        plugin::{Resolution, WorldChunk},
        Sandbox,
    },
};

pub struct DebugUiPlugin;

#[derive(Resource)]
pub struct DebugSettings {
    pub show_borders: bool,
}

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .add_plugins(FrameTimeDiagnosticsPlugin::new(10))
        .insert_resource(DebugSettings {
            show_borders: false,
        })
        .add_systems(Update, tweak_settings)
        .add_systems(Update, diagnostics_ui)
        .add_systems(
            Update,
            (draw_chunk_borders, draw_dirty_rect).run_if(should_draw_chunk_borders),
        );
    }
}

fn tweak_settings(mut settings: ResMut<DebugSettings>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::F1) {
        settings.show_borders = !settings.show_borders;
    }
}

fn diagnostics_ui(diagnostics: Res<DiagnosticsStore>, mut contexts: EguiContexts) {
    egui::Window::new("Diagnostics").show(contexts.ctx_mut(), |ui| {
        ui.label(format!(
            "FPS: {}",
            diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|fps| fps.smoothed())
                .unwrap_or(0.0) as i64
        ));
        ui.label(format!(
            "Frame time: {}ms",
            diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
                .and_then(|ms| ms.smoothed())
                .unwrap_or(0.0) as i64
        ));
    });
}

fn should_draw_chunk_borders(settings: Res<DebugSettings>) -> bool {
    settings.show_borders
}

fn draw_chunk_borders(
    mut gizmos: Gizmos,
    world_chunks: Query<&WorldChunk>,
    resolution: Res<Resolution>,
) {
    for chunk in world_chunks.iter() {
        let size = Vec2::splat(resolution.0 * CHUNK_SIZE as f32);
        let position = (Vec2::new(0.0, -1.0) + chunk.position.as_vec2()) * size;
        gizmos.rect_2d(position + size / 2.0, size, Color::srgb_u8(0, 255, 0));
    }
}

fn draw_dirty_rect(
    mut gizmos: Gizmos,
    world_chunks: Query<&WorldChunk>,
    sandbox: Res<Sandbox>,
    resolution: Res<Resolution>,
) {
    for chunk in world_chunks.iter() {
        let chunk = sandbox.get_chunk(chunk.position).unwrap();
        let rect = chunk.dirty_rect();
        if rect.is_empty() {
            continue;
        }

        let chunk_size = Vec2::splat(resolution.0 * CHUNK_SIZE as f32);
        let chunk_position = (Vec2::new(0.0, -1.0) + chunk.position.as_vec2()) * chunk_size;

        // Calculate size, ensuring it's at least 1 pixel
        let size = (rect.size().as_vec2() * resolution.0).max(Vec2::splat(1.0));

        // Calculate the top-left corner of the rect
        let rect_top_left =
            Vec2::new(rect.min.x as f32, CHUNK_SIZE as f32 - rect.max.y as f32) * resolution.0;

        // Adjust for pixel-perfect alignment and correct the 1-pixel offset
        let final_position = (chunk_position + rect_top_left).floor() + Vec2::splat(0.5) * size;

        gizmos.rect_2d(final_position, size, Color::srgb_u8(252, 115, 3));
    }
}
