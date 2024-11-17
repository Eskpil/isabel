use std::sync::Arc;

use egui::{Pos2, ViewportId};
use egui_glow::glow::{self, HasContext};
use isabel_rs::{backend::Backend, backend::Surface};

pub fn screen_size_in_pixels(width: usize, height: usize) -> egui::Vec2 {
    egui::vec2(width as f32, height as f32)
}

/// Calculate the `pixels_per_point` for a given window, given the current egui zoom factor
pub fn pixels_per_point(egui_ctx: &egui::Context, scale_factor: usize) -> f32 {
    let native_pixels_per_point = scale_factor as f32;
    let egui_zoom_factor = egui_ctx.zoom_factor();
    egui_zoom_factor * native_pixels_per_point
}

pub struct Ui {
    pub egui_ctx: egui::Context,
    gl: Arc<glow::Context>,
    painter: egui_glow::Painter,

    surface: Surface,
    viewport_info: egui::ViewportInfo,
}

impl Ui {
    pub fn new(surface: Surface) -> anyhow::Result<Self> {
        let egui_ctx = egui::Context::default();

        surface.make_current().expect("could not make current");

        let glow = Arc::new(unsafe {
            glow::Context::from_loader_function(|s| Backend::get_proc_address(s).unwrap() as _)
        });

        let painter = egui_glow::Painter::new(Arc::clone(&glow), "", None, false)?;

        Ok(Self {
            gl: glow,
            egui_ctx,
            painter,
            surface,
            viewport_info: Default::default(),
        })
    }

    pub fn run(&mut self, run_ui: impl FnMut(&egui::Context)) {
        let mut raw_input = egui::RawInput::default();

        self.viewport_info.native_pixels_per_point = Some(1.0);

        raw_input
            .viewports
            .insert(ViewportId::ROOT, self.viewport_info.clone());

        raw_input.viewport_id = ViewportId::ROOT;

        let screen_size_in_pixels = screen_size_in_pixels(200, 200);
        let screen_size_in_points = screen_size_in_pixels / pixels_per_point(&self.egui_ctx, 1);

        raw_input.screen_rect = (screen_size_in_points.x > 0.0 && screen_size_in_points.y > 0.0)
            .then(|| egui::Rect::from_min_size(Pos2::ZERO, screen_size_in_points));

        let mut full_output = self.egui_ctx.run(raw_input, run_ui);

        if !self.surface.present() {
            return;
        }

        self.surface.make_current().expect("could not make current");

        for (id, delta) in &full_output.textures_delta.set {
            self.painter.set_texture(*id, delta);
        }

        if self.surface.dimensions().is_none() {
            return;
        }

        unsafe {
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        };

        let clipped_primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let dimensions = self.surface.dimensions().unwrap().map(|v| v as u32);
        self.painter.paint_primitives(
            dimensions,
            full_output.pixels_per_point,
            &clipped_primitives,
        );

        for id in full_output.textures_delta.free.drain(..) {
            self.painter.free_texture(id);
        }

        self.surface.swap_buffers().expect("could not swap buffers");
        self.surface
            .clear_current()
            .expect("could not clear current");
    }
}
