mod label;
mod tick;

use iced::{
    Font, Point, alignment,
    widget::canvas::{self, Path, Stroke},
};
pub use label::Labels;
pub use tick::Tick;

use super::cartesian::Plane;

pub struct Axis<'a> {
    pub color: iced::Color,
    pub width: f32,
    label: Labels<'a>,
    tick: Tick,
}

impl<'a> Axis<'a> {
    pub fn color(mut self, color: iced::Color) -> Self {
        self.color = color;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn draw(&self, frame: &mut canvas::Frame, plane: &Plane) {
        let bounds = frame.size();

        let mut scaled_bottom_left = plane.scale_to_cartesian(plane.bottom_left());
        let mut scaled_bottom_right = plane.scale_to_cartesian(plane.bottom_right());

        let label_height = 10.0;
        if scaled_bottom_left.x > bounds.height - label_height {
            // TODO minus label height
            scaled_bottom_left.x = bounds.width - label_height;
            scaled_bottom_right.x = bounds.width - label_height;
        }

        frame.stroke(
            &Path::line(scaled_bottom_left, scaled_bottom_right),
            Stroke::default()
                .with_width(self.width)
                .with_color(self.color),
        );

        // ticks
        let tick_width = plane.x.length / self.tick.amount as f32;
        let mut draw_x_tick = |x| {
            let x_scaled = plane.scale_to_cartesian_x(x);
            let y_scaled = plane.scale_to_cartesian_y(0.0);

            let half_tick_height = self.tick.height / 2.0;
            let x_start = Point {
                x: x_scaled,
                y: y_scaled - half_tick_height,
            };
            let x_end = Point {
                x: x_scaled,
                y: y_scaled + half_tick_height,
            };

            frame.stroke(
                &Path::line(x_start, x_end),
                Stroke::default()
                    .with_width(self.tick.width)
                    .with_color(self.tick.color),
            );

            let label = self
                .label
                .format
                .map_or_else(|| format!("{x}"), |fmt| fmt(&x));

            frame.fill_text(canvas::Text {
                content: label,
                size: self.label.font_size.unwrap_or(12.into()),
                position: Point {
                    x: x_scaled,
                    // TODO remove magic number,
                    y: y_scaled + 8.0,
                },
                // TODO use theme
                color: self.label.color.unwrap_or(iced::Color::WHITE),
                // TODO edge case center tick
                align_x: iced::widget::text::Alignment::Center,
                align_y: alignment::Vertical::Top,
                font: Font::MONOSPACE,
                ..canvas::Text::default()
            });
        };

        let left = (plane.x.min / tick_width).ceil() as i32;
        for i in left..0 {
            draw_x_tick(i as f32 * tick_width);
        }

        let right = (plane.x.max / tick_width).floor() as i32;
        for i in 0..=right {
            draw_x_tick(i as f32 * tick_width);
        }
    }
}

impl<'a> Default for Axis<'a> {
    fn default() -> Self {
        Self {
            // use color from theme
            color: iced::Color::WHITE,
            width: 1.0,
            label: Labels::default(),
            tick: Tick::default(),
        }
    }
}
