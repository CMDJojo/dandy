use crate::pos2::Pos2;
use crate::Drawer;
use egui::{emath, Align2, Color32, FontId, Painter, Rounding, Stroke};

static DEFAULT_COLOR: Color32 = Color32::from_rgb(0, 255, 0);

pub struct EguiDrawer<'a> {
    painter: &'a Painter,
    color: Color32,
}

impl<'a> EguiDrawer<'a> {
    pub fn new(painter: &'a Painter) -> Self {
        Self {
            painter,
            color: DEFAULT_COLOR,
        }
    }
}

impl Drawer for EguiDrawer<'_> {
    fn start_drawing(&mut self) {}

    fn finish_drawing(&mut self) {}

    fn draw_circle(&mut self, pos: Pos2, radius: f32, thickness: f32) {
        self.painter
            .circle_stroke(pos.into(), radius, Stroke::new(thickness, self.color));
    }

    fn draw_centered_text(&mut self, pos: Pos2, text: &str) {
        self.painter.text(
            pos.into(),
            Align2::CENTER_CENTER,
            text,
            FontId::default(),
            self.color,
        );
    }

    fn draw_rect(&mut self, upper_left: Pos2, size: Pos2) {
        self.painter.rect_filled(
            emath::Rect::from_min_size(upper_left.into(), egui::Pos2::from(size).to_vec2()),
            Rounding::ZERO,
            self.color,
        );
    }

    fn draw_line(&mut self, from: Pos2, to: Pos2, thickness: f32) {
        self.painter
            .line_segment([from.into(), to.into()], Stroke::new(thickness, self.color))
    }

    fn set_color(&mut self, rgb: [u8; 3]) {
        let [r, g, b] = rgb;
        self.color = Color32::from_rgb(r, g, b);
    }
}
