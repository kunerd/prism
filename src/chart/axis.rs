mod label;
mod tick;

pub use label::Labels;
pub use tick::Tick;

pub struct Axis {
    pub color: iced::Color,
    pub width: f32,
    // TODO limits
}

impl Axis {
    pub fn color(mut self, color: iced::Color) -> Self {
        self.color = color;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            // use color from theme
            color: iced::Color::WHITE,
            width: 1.0,
        }
    }
}
