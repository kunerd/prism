#[derive(Default)]
pub struct Labels<'a> {
    pub color: Option<iced::Color>,
    pub font_size: Option<iced::Pixels>,
    pub format: Option<&'a dyn Fn(&f32) -> String>, // TODO:
                                                    // alignment
                                                    // limits
                                                    // uppercase    -- Make labels uppercase
                                                    // rotate 90    -- Rotate labels

                                                    // CA.alignRight   -- Anchor labels to the right
                                                    // CA.alignLeft    -- Anchor labels to the left

                                                    // CA.moveUp 5     -- Move 5 SVG units up
                                                    // CA.moveDown 5   -- Move 5 SVG units down
                                                    // CA.moveLeft 5   -- Move 5 SVG units left
                                                    // CA.moveRight 5  -- Move 5 SVG units right

                                                    // CA.amount 15   -- Change amount of ticks
                                                    // , CA.flip        -- Flip to opposite direction
                                                    // CA.withGrid    -- Add grid line by each label.

                                                    // CA.ints            -- Add ticks at "nice" ints
                                                    // CA.times Time.utc  -- Add ticks at "nice" times
}

impl<'a> Labels<'a> {
    pub fn color(mut self, color: iced::Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn font_size(mut self, font_size: impl Into<iced::Pixels>) -> Self {
        self.font_size = Some(font_size.into());
        self
    }

    pub fn format(mut self, format: &'a dyn Fn(&f32) -> String) -> Self {
        self.format = Some(format);
        self
    }
}
