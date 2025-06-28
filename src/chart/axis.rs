mod label;
mod tick;

pub use label::Labels;
pub use tick::Tick;

use super::cartesian::Plane;

use iced::{
    Font, Point, alignment,
    widget::canvas::{self, Path, Stroke},
};

pub struct Axis<'a> {
    alignment: Alignment,
    color: iced::Color,
    width: f32,
    label: Labels<'a>,
    tick: Tick,
}

pub enum Alignment {
    Horizontal,
    Vertical,
}

impl<'a> Axis<'a> {
    pub fn new(alignment: Alignment) -> Self {
        Self {
            alignment,
            color: iced::Color::WHITE,
            width: 1.0,
            label: Labels::default(),
            tick: Tick::default(),
        }
    }
    pub fn color(mut self, color: iced::Color) -> Self {
        self.color = color;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub(super) fn draw(&self, frame: &mut canvas::Frame, plane: &Plane) {
        let (start, end) = match self.alignment {
            Alignment::Horizontal => (plane.bottom_left(), plane.bottom_right()),
            Alignment::Vertical => (plane.bottom_center(), plane.top_center()),
        };

        // TODO: fix clamping
        // let bounds = frame.size();
        // let label_height = 10.0;
        // if scaled_bottom_left.x > bounds.height - label_height {
        //     scaled_bottom_left.x = bounds.width - label_height;
        //     scaled_bottom_right.x = bounds.width - label_height;
        // }

        let start = plane.scale_to_cartesian(start);
        let end = plane.scale_to_cartesian(end);

        frame.stroke(
            &Path::line(start, end),
            Stroke::default()
                .with_width(self.width)
                .with_color(self.color),
        );

        // ticks
        let length = match self.alignment {
            Alignment::Horizontal => plane.x.length,
            Alignment::Vertical => plane.y.length,
        };

        let tick_distance = length / self.tick.amount as f32;

        let mut draw_tick = |x, y| {
            let x_scaled = plane.scale_to_cartesian_x(x);
            let y_scaled = plane.scale_to_cartesian_y(y);

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

            let label = match self.alignment {
                Alignment::Horizontal => x,
                Alignment::Vertical => y,
            };

            let label = self
                .label
                .format
                .map_or_else(|| format!("{}", label), |fmt| fmt(&label));

            frame.fill_text(canvas::Text {
                content: label,
                size: self.label.font_size.unwrap_or(12.into()),
                position: Point {
                    x: x_scaled,
                    y: y_scaled,
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

        let axis = match self.alignment {
            Alignment::Horizontal => &plane.x,
            Alignment::Vertical => &plane.y,
        };

        let start = (axis.min / tick_distance).ceil() as i32;
        for i in start..0 {
            match self.alignment {
                Alignment::Horizontal => draw_tick(i as f32 * tick_distance, 0.0),
                Alignment::Vertical => draw_tick(0.0, i as f32 * tick_distance),
            }
        }

        let end = (axis.max / tick_distance).floor() as i32;
        for i in 0..=end {
            match self.alignment {
                Alignment::Horizontal => draw_tick(i as f32 * tick_distance, 0.0),
                Alignment::Vertical => draw_tick(0.0, i as f32 * tick_distance),
            }
        }
    }

    pub(crate) fn labels(&mut self, labels: Labels<'a>) {
        self.label = labels;
    }
}
