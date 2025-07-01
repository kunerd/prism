use iced::widget::canvas::{Frame, Path, Stroke};

use super::{Alignment, Ticks};

pub struct Tick {
    position: iced::Point,
}

impl Tick {
    pub fn new(position: iced::Point) -> Self {
        Self { position }
    }

    pub fn draw(&self, frame: &mut Frame, alignment: Alignment, ticks: &Ticks) {
        let half_tick_length = ticks.length / 2.0;

        let (x_start, x_end) = match alignment {
            Alignment::Horizontal => (
                iced::Vector {
                    x: 0.0,
                    y: -half_tick_length,
                },
                iced::Vector {
                    x: 0.0,
                    y: half_tick_length,
                },
            ),
            Alignment::Vertical => (
                iced::Vector {
                    x: -half_tick_length,
                    y: 0.0,
                },
                iced::Vector {
                    x: half_tick_length,
                    y: 0.0,
                },
            ),
        };

        let start = self.position + x_start;
        let end = self.position + x_end;

        frame.stroke(
            &Path::line(start, end),
            Stroke::default()
                .with_width(ticks.width)
                .with_color(ticks.color),
        );
    }
}
