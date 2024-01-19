use egui::{Align2, Color32, emath, FontId, Rounding, Stroke};
use dandy_draw::Drawer;

pub struct EguiDrawer<'a> {
    pub painter: &'a egui::Painter,
}

static DRAW_COLOR: Color32 = Color32::from_rgb(0, 255, 0);

impl Drawer for EguiDrawer<'_> {
    fn start_drawing(&mut self) {}

    fn finish_drawing(&mut self) {}

    fn draw_circle(&mut self, pos: (f32, f32), radius: f32, thickness: f32) {
        self.painter.circle_stroke(pos.into(), radius, Stroke::new(
            thickness,
            DRAW_COLOR,
        ));
    }

    fn draw_centered_text(&mut self, pos: (f32, f32), text: &str) {
        self.painter.text(pos.into(), Align2::CENTER_CENTER, text, FontId::default(), DRAW_COLOR);
    }

    fn draw_rect(&mut self, upper_left: (f32, f32), size: (f32, f32)) {
        self.painter.rect_filled(
            emath::Rect::from_min_size(
                upper_left.into(),
                size.into(),
            ),
            Rounding::ZERO,
            DRAW_COLOR,
        );
    }

    fn draw_line(&mut self, from: (f32, f32), to: (f32, f32), thickness: f32) {
        self.painter.line_segment([from.into(), to.into()], Stroke::new(
            thickness,
            DRAW_COLOR,
        ))
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
