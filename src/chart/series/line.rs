use std::ops::RangeInclusive;

use crate::chart::cartesian::Plane;

use super::Series;

use iced::{
    Color, Point, Vector,
    widget::canvas::{self, Path, Stroke, path::lyon_path::geom::euclid::Transform2D},
};

#[derive(Clone)]
pub struct LineSeries<Data> {
    pub data: Data,
    pub color: Color,
}

impl<Data> LineSeries<Data> {
    pub fn new(data: Data) -> Self {
        Self {
            data,
            color: Color::BLACK,
        }
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }
}

impl<Id, Data> Series<Id> for LineSeries<Data>
where
    Data: IntoIterator + Clone,
    Data::Item: Into<(f32, f32)>,
{
    fn draw(&self, frame: &mut canvas::Frame, plane: &Plane) {
        frame.with_save(|frame| {
            frame.translate(Vector::new(plane.x.margin_min, plane.x.margin_min));
            frame.scale_nonuniform(Vector::new(plane.x.scale, plane.y.scale));
            frame.translate(Vector::new(-plane.x.min, plane.y.max));

            let mut iter = self
                .data
                .clone()
                .into_iter()
                .map(Into::into)
                .filter(|(x, y)| {
                    x >= &plane.x.min && x <= &plane.x.max && y >= &plane.y.min && y <= &plane.y.max
                });

            let path = Path::new(|b| {
                if let Some(p) = iter.next() {
                    b.move_to(Point { x: p.0, y: p.1 });
                    iter.fold(b, |acc, p| {
                        acc.line_to(Point { x: p.0, y: p.1 });
                        acc
                    });
                }
            });

            frame.stroke(
                &path.transform(&Transform2D::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0)),
                Stroke::default().with_width(2.0).with_color(self.color),
            );
        })
    }

    fn x_range(&self) -> RangeInclusive<f32> {
        let x_min_cur = f32::INFINITY;
        let x_max_cur = f32::NEG_INFINITY;

        let (x_min, x_max) = {
            self.data
                .clone()
                .into_iter()
                .map(Into::into)
                .fold((x_min_cur, x_max_cur), |(x_min, x_max), (cur_x, _)| {
                    (x_min.min(cur_x), x_max.max(cur_x))
                })
        };

        x_min..=x_max
    }

    fn y_range(&self) -> RangeInclusive<f32> {
        let y_min_cur = f32::INFINITY;
        let y_max_cur = f32::NEG_INFINITY;

        let (y_min, y_max) = {
            self.data
                .clone()
                .into_iter()
                .map(Into::into)
                .fold((y_min_cur, y_max_cur), |(y_min, y_max), (_, cur_y)| {
                    (y_min.min(cur_y), y_max.max(cur_y))
                })
        };

        y_min..=y_max
    }
}
