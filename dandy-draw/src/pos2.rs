use std::ops::{Add, Mul, Neg, Sub};

pub fn pos2(x: f32, y: f32) -> Pos2 {
    Pos2 { x, y }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pos2 {
    pub x: f32,
    pub y: f32,
}

impl Pos2 {
    pub fn with_x(self, x: f32) -> Self {
        Self { x, y: self.y }
    }

    pub fn with_y(self, y: f32) -> Self {
        Self { x: self.x, y }
    }

    pub fn x(x: f32) -> Self {
        Self { x, y: 0.0 }
    }

    pub fn y(y: f32) -> Self {
        Self { x: 0.0, y }
    }
}

impl Add for Pos2 {
    type Output = Pos2;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Pos2 {
    type Output = Pos2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for Pos2 {
    type Output = Pos2;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Pos2> for Pos2 {
    type Output = Pos2;

    fn mul(self, rhs: Pos2) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Neg for Pos2 {
    type Output = Pos2;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y
        }
    }
}

impl From<(f32, f32)> for Pos2 {
    fn from((x, y): (f32, f32)) -> Self {
        pos2(x, y)
    }
}

impl From<(f64, f64)> for Pos2 {
    fn from((x, y): (f64, f64)) -> Self {
        pos2(x as f32, y as f32)
    }
}

impl From<Pos2> for (f32, f32) {
    fn from(value: Pos2) -> Self {
        (value.x, value.y)
    }
}

#[cfg(feature = "egui")]
impl From<Pos2> for egui::Pos2 {
    fn from(value: Pos2) -> Self {
        egui::pos2(value.x, value.y)
    }
}
