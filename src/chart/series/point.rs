use std::ops::RangeInclusive;

use iced::{
    Color, Point,
    widget::canvas::{self, Path, Stroke},
};

use crate::chart::{cartesian::Plane, items};

use super::Series;

type StyleFn<'a, Item> = Box<dyn Fn(usize, &Item) -> Style + 'a>;

pub struct PointSeries<'a, SeriesId, Item, Data>
where
    SeriesId: Clone,
    Data: IntoIterator<Item = Item>,
{
    pub id: Option<SeriesId>,
    pub data: Data,
    pub color: Color,
    x_fn: Option<&'a dyn Fn(&Item) -> f32>,
    y_fn: Option<&'a dyn Fn(&Item) -> f32>,
    collision_box: Option<iced::Rectangle>,
    style: Style,
    pub style_fn: Option<StyleFn<'a, Item>>,
}

#[derive(Debug, Clone)]
pub struct Style {
    pub color: Option<iced::Color>,
    pub border_color: Option<iced::Color>,
    pub border: f32,
    pub radius: f32,
}

impl<'a, ID, Item, Data> PointSeries<'a, ID, Item, Data>
where
    ID: Clone,
    Data: IntoIterator<Item = Item>,
    //Data: IntoIterator + Clone,
    //Data::Item: Into<(f32, f32)>,
{
    pub fn new(data: Data) -> Self {
        Self {
            id: None,
            data,
            x_fn: None,
            y_fn: None,
            color: Color::BLACK,
            collision_box: None,
            style: Style::default(),
            style_fn: None,
        }
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    pub fn collision_box(mut self, collision_box: impl Into<iced::Rectangle>) -> Self {
        self.collision_box = Some(collision_box.into());
        self
    }

    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    pub fn style_for_each(mut self, style_fn: impl Fn(usize, &Item) -> Style + 'a) -> Self {
        self.style_fn = Some(Box::new(style_fn));
        self
    }

    pub fn with_id(mut self, id: ID) -> Self {
        self.id = Some(id);
        self
    }

    pub fn x(mut self, x_fn: &'a dyn Fn(&Item) -> f32) -> Self {
        self.x_fn = Some(x_fn);
        self
    }
    pub fn y(mut self, y_fn: &'a dyn Fn(&Item) -> f32) -> Self {
        self.y_fn = Some(y_fn);
        self
    }
}

impl<Id, Item, Data> Series<Id> for PointSeries<'_, Id, Item, Data>
where
    Id: Clone,
    Data: IntoIterator<Item = Item> + Clone,
    Item: Into<(f32, f32)>,
{
    fn draw(&self, frame: &mut canvas::Frame, plane: &Plane) {
        for (index, item) in self.data.clone().into_iter().enumerate() {
            let style = self
                .style_fn
                .as_ref()
                .map(|func| func(index, &item))
                .unwrap_or_default();

            let x = self.x_fn.as_ref().map(|f| f(&item));
            let y = self.y_fn.as_ref().map(|f| f(&item));

            let p = item.into();
            let point = Point {
                x: plane.scale_to_cartesian_x(x.unwrap_or(p.0)),
                y: plane.scale_to_cartesian_y(y.unwrap_or(p.1)),
            };

            let color = style.color.unwrap_or(self.color);
            let border_color = style.border_color.unwrap_or(self.color);

            let path = &Path::circle(point, style.radius);

            frame.fill(
                path,
                canvas::Fill {
                    style: canvas::Style::Solid(color),
                    ..Default::default()
                },
            );

            frame.stroke(
                path,
                Stroke::default()
                    .with_width(style.border)
                    .with_color(border_color),
            );
        }
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

    fn id(&self) -> Option<Id> {
        self.id.clone()
    }

    fn collision_box(&self) -> Option<iced::Rectangle> {
        let style = Style::default();
        self.collision_box
            .or_else(|| Some(iced::Rectangle::with_radius(style.radius)))
    }

    fn items(&self) -> Option<(Id, Vec<items::Entry<usize>>)> {
        let id = self.id.clone()?;

        let items: Vec<_> = self
            .data
            .clone()
            .into_iter()
            .map(Into::into)
            .enumerate()
            .map(|(index, (x, y))| items::Entry::new(index, iced::Point::new(x, y)))
            .collect();

        Some((id, items))
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            color: None,
            border_color: None,
            border: 2.0,
            radius: 5.0,
        }
    }
}
