use iced::{
    Font, Pixels,
    advanced::{
        graphics::{geometry, text::Paragraph},
        text::{self, Paragraph as _},
    },
    alignment,
    widget::{
        canvas::{self, Frame},
        text::{Fragment, IntoFragment},
    },
};

use super::{Alignment, Labels};

pub(crate) struct Label<'a> {
    content: Fragment<'a>,
    bounds: iced::Size,
}

impl<'a> Label<'a> {
    pub(crate) fn new(content: impl IntoFragment<'a>, font_size: impl Into<Pixels>) -> Self {
        let content = content.into_fragment();
        let bounds = min_bounds(content.as_ref(), font_size.into());

        Self { content, bounds }
    }

    pub(crate) fn min_width(&self) -> f32 {
        self.bounds.width
    }

    pub(crate) fn min_height(&self) -> f32 {
        self.bounds.height
    }

    pub(crate) fn draw<Renderer: geometry::Renderer>(
        &self,
        frame: &mut Frame<Renderer>,
        pos: iced::Point,
        alignment: Alignment,
        config: &Labels,
    ) {
        let (align_x, align_y) = match alignment {
            Alignment::Horizontal => (text::Alignment::Center, alignment::Vertical::Top),
            Alignment::Vertical => (text::Alignment::Right, alignment::Vertical::Center),
        };

        frame.fill_text(canvas::Text {
            content: self.content.to_string(),
            size: config.font_size.unwrap_or(12.into()),
            position: pos,
            color: config.color.unwrap_or(iced::Color::WHITE),
            align_x,
            align_y,
            font: Font::MONOSPACE,
            ..canvas::Text::default()
        });
    }
}

fn min_bounds(content: &str, font_size: Pixels) -> iced::Size {
    let text = iced::advanced::text::Text {
        content,
        size: font_size,
        line_height: text::LineHeight::default(),
        bounds: iced::Size::INFINITY,
        font: Font::MONOSPACE,
        align_x: iced::advanced::text::Alignment::Right,
        align_y: alignment::Vertical::Center,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::default(),
    };

    let paragraph = Paragraph::with_text(text);
    paragraph.min_bounds()
}
