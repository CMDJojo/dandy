use std::f64::consts::PI;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, wasm_bindgen::JsCast};
use crate::Drawer;

pub struct CanvasDrawer {
    context: CanvasRenderingContext2d,
}

impl CanvasDrawer {
    pub fn new(context: CanvasRenderingContext2d) -> Self {
        context.set_text_align("center");
        context.set_text_baseline("middle");
        Self {
            context
        }
    }

    pub fn from_element(canvas: HtmlCanvasElement) -> Option<Self> {
        let context = canvas.get_context("2d").ok()??.dyn_into().ok()?;
        Some(Self {
            context
        })
    }
}

impl Drawer for CanvasDrawer {
    fn start_drawing(&mut self) {
        self.context.begin_path();
    }

    fn finish_drawing(&mut self) {
        self.context.close_path();
    }

    fn draw_circle(&mut self, pos: (f32, f32), radius: f32, thickness: f32) {
        self.context.set_line_width(thickness as f64);
        self.context.arc(pos.0 as f64, pos.1 as f64, radius as f64, 0.0, 2.0 * PI).unwrap();
    }

    fn draw_centered_text(&mut self, pos: (f32, f32), text: &str) {
        self.context.fill_text(text, pos.0 as f64, pos.1 as f64).unwrap();
    }

    fn draw_rect(&mut self, upper_left: (f32, f32), size: (f32, f32)) {
        self.context.fill_rect(upper_left.0 as f64, upper_left.1 as f64, size.0 as f64, size.1 as f64);
    }

    fn draw_line(&mut self, from: (f32, f32), to: (f32, f32), thickness: f32) {
        self.context.set_line_width(thickness as f64);
        self.context.move_to(from.0 as f64, from.1 as f64);
        self.context.line_to(to.0 as f64, to.1 as f64);
    }
}
