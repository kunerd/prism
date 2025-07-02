mod line;
pub mod point;

pub use line::LineSeries;
pub use point::PointSeries;

use super::{
    axis::{self},
    cartesian::Plane,
    items,
};

use iced::{
    advanced::graphics::geometry,
    widget::canvas::{self},
};

use std::ops::RangeInclusive;

pub trait Series<SeriesId, Renderer, ItemId = usize>
where
    Renderer: geometry::Renderer,
{
    fn draw(&self, frame: &mut canvas::Frame<Renderer>, plane: &Plane, x_scale: &axis::Scale);
    fn id(&self) -> Option<SeriesId> {
        None
    }
    fn collision_box(&self) -> Option<iced::Rectangle> {
        None
    }
    fn items(&self) -> Option<(SeriesId, Vec<items::Entry<ItemId>>)> {
        None
    }
    fn x_range(&self) -> RangeInclusive<f32>;
    fn y_range(&self) -> RangeInclusive<f32>;
}

pub fn line_series<Data>(data: Data) -> LineSeries<Data> {
    LineSeries::new(data)
}

pub fn point_series<'a, Id, Item, Data>(data: Data) -> PointSeries<'a, Id, Item, Data>
where
    Id: Clone,
    Data: IntoIterator<Item = Item>,
{
    PointSeries::new(data)
}
