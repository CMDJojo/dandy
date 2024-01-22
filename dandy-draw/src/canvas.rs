use crate::pos2::Pos2;
use crate::Drawer;
use std::f64::consts::PI;
use web_sys::{wasm_bindgen::JsCast, CanvasRenderingContext2d, HtmlCanvasElement};

pub struct CanvasDrawer {
    context: CanvasRenderingContext2d,
}

impl CanvasDrawer {
    pub fn new(context: CanvasRenderingContext2d) -> Self {
        context.set_text_align("center");
        context.set_text_baseline("middle");
        Self { context }
    }

    pub fn from_element(canvas: HtmlCanvasElement) -> Option<Self> {
        let context = canvas.get_context("2d").ok()??.dyn_into().ok()?;
        Some(Self { context })
    }
}

impl Drawer for CanvasDrawer {
    fn start_drawing(&mut self) {
        self.context.begin_path();
    }

    fn finish_drawing(&mut self) {
        self.context.close_path();
    }

    fn draw_circle(&mut self, pos: Pos2, radius: f32, thickness: f32) {
        self.context.set_line_width(thickness as f64);
        self.context
            .arc(pos.x as f64, pos.y as f64, radius as f64, 0.0, 2.0 * PI)
            .unwrap();
    }

    fn draw_centered_text(&mut self, pos: Pos2, text: &str) {
        self.context
            .fill_text(text, pos.x as f64, pos.y as f64)
            .unwrap();
    }

    fn draw_rect(&mut self, upper_left: Pos2, size: Pos2) {
        self.context.fill_rect(
            upper_left.x as f64,
            upper_left.y as f64,
            size.x as f64,
            size.y as f64,
        );
    }

    fn draw_line(&mut self, from: Pos2, to: Pos2, thickness: f32) {
        self.context.set_line_width(thickness as f64);
        self.context.move_to(from.x as f64, from.y as f64);
        self.context.line_to(to.x as f64, to.y as f64);
    }
}
