use std::{f32, ops::RangeInclusive};

pub struct Plane {
    pub x: Axis,
    pub y: Axis,
}

impl Plane {
    pub fn bottom_center(&self) -> iced::Point {
        iced::Point {
            x: 0.0,
            y: self.y.min,
        }
    }

    pub fn top_center(&self) -> iced::Point {
        iced::Point {
            x: 0.0,
            y: self.y.max,
        }
    }

    pub fn bottom_left(&self) -> iced::Point {
        iced::Point {
            x: self.x.min,
            y: 0.0,
        }
    }

    pub fn bottom_right(&self) -> iced::Point {
        iced::Point {
            x: self.x.max,
            y: 0.0,
        }
    }

    pub fn scale_to_cartesian_x(&self, value: f32) -> f32 {
        let mut result = value - self.x.min;
        result *= self.x.scale;
        result += self.x.margin_min;

        result
    }

    pub fn scale_to_cartesian_y(&self, value: f32) -> f32 {
        let mut result = -value + self.y.max;
        result *= self.y.scale;
        result += self.y.margin_max;

        result
    }

    pub fn get_cartesian(&self, pos: iced::Point) -> iced::Point {
        let mut point =
            pos * iced::Transformation::translate(-self.x.margin_min, -self.y.margin_min);
        point.x /= self.x.scale;
        point.y /= self.y.scale;
        let mut point = point * iced::Transformation::translate(self.x.min, -self.y.max);
        point.y = -point.y;

        point
    }

    pub fn get_offset(&self, pos: iced::Point) -> iced::Point {
        let pos = self.get_cartesian(pos);

        iced::Point::new(
            pos.x - (self.x.min + self.x.length / 2.0),
            pos.y - (self.y.min + self.y.length / 2.0),
        )
    }

    pub fn scale_to_cartesian(&self, point: iced::Point) -> iced::Point {
        iced::Point {
            x: self.scale_to_cartesian_x(point.x),
            y: self.scale_to_cartesian_y(point.y),
        }
    }
}

pub struct Axis {
    pub length: f32,
    pub scale: f32,
    pub margin_min: f32,
    pub margin_max: f32,
    pub min: f32,
    pub max: f32,
}

impl Axis {
    pub fn new(range: &RangeInclusive<f32>, margin_min: f32, margin_max: f32, width: f32) -> Self {
        let length = -range.start() + range.end();
        let margin = margin(margin_min, margin_max);
        let scale = (width - margin) / length;

        let min = *range.start();
        let max = *range.end();

        Self {
            length,
            scale,
            margin_min,
            margin_max,
            min,
            max,
        }
    }
}

fn margin(min: f32, max: f32) -> f32 {
    min + max
}
