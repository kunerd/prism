pub struct Tick {
    pub color: iced::Color,
    pub height: f32,
    pub width: f32,
    pub amount: usize,
    //flip
    //noGrid
    //ints
    //times
    //limits: RangeInclusive<T>
}

impl Tick {
    pub fn color(mut self, color: iced::Color) -> Self {
        self.color = color;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn amount(mut self, amount: usize) -> Self {
        self.amount = amount;
        self
    }
}

impl Default for Tick {
    fn default() -> Self {
        Self {
            // use color from theme
            color: iced::Color::WHITE,
            height: 5.0,
            width: 1.0,
            amount: 10,
        }
    }
}
