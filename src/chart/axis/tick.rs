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

        // let label = match self.alignment {
        //     Alignment::Horizontal => 10_f32.powf(x),
        //     Alignment::Vertical => y,
        // };

        // let (align_x, align_y) = match self.alignment {
        //     Alignment::Horizontal => (text::Alignment::Center, alignment::Vertical::Top),
        //     Alignment::Vertical => (text::Alignment::Right, alignment::Vertical::Center),
        // };

        // frame.fill_text(canvas::Text {
        //     content: label,
        //     size: self.label.font_size.unwrap_or(12.into()),
        //     position: Point {
        //         x: x_scaled,
        //         y: y_scaled,
        //     },
        //     color: self.label.color.unwrap_or(iced::Color::WHITE),
        //     align_x,
        //     align_y,
        //     font: Font::MONOSPACE,
        //     ..canvas::Text::default()
        // });
    }
}
