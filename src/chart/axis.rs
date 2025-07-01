mod label;
mod labels;
mod tick;
mod ticks;

use label::Label;
pub use labels::Labels;
use tick::Tick;
pub use ticks::Ticks;

use super::cartesian::Plane;

use iced::widget::canvas::{self, Path, Stroke};

pub struct Axis<'a> {
    scale: Scale,
    alignment: Alignment,
    color: iced::Color,
    width: f32,
    labels: Labels<'a>,
    ticks: Ticks,
    x_tick_marks: Vec<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Horizontal,
    Vertical,
}

pub enum Scale {
    Linear,
    Log,
}

impl<'a> Axis<'a> {
    pub fn new(alignment: Alignment) -> Self {
        Self {
            scale: Scale::Linear,
            alignment,
            color: iced::Color::WHITE,
            width: 1.0,
            labels: Labels::default(),
            ticks: Ticks::default(),
            x_tick_marks: vec![],
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

    pub fn scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    pub fn x_tick_marks(mut self, x_tick_marks: Vec<f32>) -> Self {
        self.x_tick_marks = x_tick_marks;
        self
    }

    pub(super) fn draw(&self, frame: &mut canvas::Frame, plane: &Plane) {
        let bounds = frame.size();

        let (start, end) = match self.alignment {
            Alignment::Horizontal => (plane.bottom_left(), plane.bottom_right()),
            Alignment::Vertical => (plane.bottom_center(), plane.top_center()),
        };

        let mut start = plane.scale_to_cartesian(start);
        let mut end = plane.scale_to_cartesian(end);

        // ticks
        let length = match self.alignment {
            Alignment::Horizontal => plane.x.length,
            Alignment::Vertical => plane.y.length,
        };
        let axis = match self.alignment {
            Alignment::Horizontal => &plane.x,
            Alignment::Vertical => &plane.y,
        };

        let tick_distance = length / self.ticks.amount as f32;
        let start_tick = (axis.min / tick_distance).ceil() as i32;
        let end_tick = (axis.max / tick_distance).floor() as i32;
        let (pos, labels): (Vec<_>, Vec<_>) = (start_tick..0)
            .into_iter()
            .chain(1..=end_tick)
            .map(|i| {
                let pos = match self.scale {
                    Scale::Linear => i as f32 * tick_distance,
                    Scale::Log => ((i as f32 * tick_distance).log10() / length.log10()) * length,
                };

                let content = self
                    .labels
                    .format
                    .map_or_else(|| format!("{}", pos), |fmt| fmt(&pos));

                let label = Label::new(
                    content,
                    self.labels
                        .font_size
                        .unwrap_or_else(|| Labels::DEFAULT_FONT_SIZE.into()),
                );

                (pos, label)
            })
            .collect();

        let max_label_width = labels
            .iter()
            .max_by(|acc, e| acc.min_width().total_cmp(&e.min_width()))
            .unwrap();
        let max_label_width = max_label_width.min_width();

        let max_label_height = labels
            .iter()
            .max_by(|acc, e| acc.min_height().total_cmp(&e.min_height()))
            .unwrap();
        let max_label_height = max_label_height.min_height();

        match self.alignment {
            Alignment::Horizontal => {
                if start.y <= 0.0 + self.ticks.length {
                    start.y = self.ticks.length;
                    end.y = self.ticks.length;
                } else if start.y >= bounds.height - max_label_height {
                    start.y = bounds.height - max_label_height;
                    end.y = bounds.height - max_label_height;
                }
            }
            Alignment::Vertical => {
                if start.x - max_label_width <= 0.0 {
                    start.x = max_label_width + self.width;
                    end.x = max_label_width + self.width;
                } else if start.x >= bounds.width - self.ticks.length {
                    start.x = bounds.width - self.ticks.length;
                    end.x = bounds.width - self.ticks.length;
                }
            }
        }

        frame.stroke(
            &Path::line(start, end),
            Stroke::default()
                .with_width(self.width)
                .with_color(self.color),
        );

        if self.x_tick_marks.is_empty() {
            pos.into_iter().zip(labels).for_each(|(pos, label)| {
                let pos = match self.alignment {
                    Alignment::Horizontal => {
                        iced::Point::new(plane.scale_to_cartesian_x(pos), start.y)
                    }
                    Alignment::Vertical => {
                        iced::Point::new(start.x, plane.scale_to_cartesian_y(pos))
                    }
                };

                Tick::new(pos).draw(frame, self.alignment, &self.ticks);
                label.draw(frame, pos, self.alignment, &self.labels);
            });
        }

        self.x_tick_marks.iter().for_each(|tm| {
            let pos = match self.scale {
                Scale::Linear => *tm as f32,
                Scale::Log => {
                    if *tm == 0.0 {
                        0.0
                    } else {
                        (tm.log10() / axis.max.log10()) * axis.max
                    }
                }
            };

            let content = self
                .labels
                .format
                .map_or_else(|| format!("{}", tm), |fmt| fmt(&tm));

            let label = Label::new(
                content,
                self.labels
                    .font_size
                    .unwrap_or_else(|| Labels::DEFAULT_FONT_SIZE.into()),
            );

            let pos = match self.alignment {
                Alignment::Horizontal => iced::Point::new(plane.scale_to_cartesian_x(pos), start.y),
                Alignment::Vertical => iced::Point::new(start.x, plane.scale_to_cartesian_y(pos)),
            };

            Tick::new(pos).draw(frame, self.alignment, &self.ticks);
            label.draw(frame, pos, self.alignment, &self.labels);
        });
    }

    pub(crate) fn labels(&mut self, labels: Labels<'a>) {
        self.labels = labels;
    }
}
