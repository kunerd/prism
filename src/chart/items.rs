use iced::Rectangle;
use ordered_float::OrderedFloat;

use std::collections::BTreeMap;

type BTreeMapFloat<V> = BTreeMap<OrderedFloat<f32>, V>;

pub struct Entry<Id> {
    id: Id,
    location: iced::Point,
}

impl<Id> Entry<Id>
where
    Id: Clone,
{
    pub fn new(id: Id, location: iced::Point) -> Self {
        Self { id, location }
    }
}

pub struct Items<SeriesId, ItemId>(BTreeMapFloat<BTreeMapFloat<(SeriesId, ItemId)>>);

impl<SeriesId, ItemId> Items<SeriesId, ItemId>
where
    SeriesId: Clone,
    ItemId: Clone + std::fmt::Debug,
{
    pub fn add_series(&mut self, id: SeriesId, series: &[Entry<ItemId>]) {
        for entry in series.iter() {
            let point = entry.location;

            self.0
                .entry(OrderedFloat(point.x))
                .or_default()
                .insert(OrderedFloat(point.y), (id.clone(), entry.id.clone()));
        }
    }

    pub fn collision(&self, rect: Rectangle) -> Vec<(SeriesId, ItemId)> {
        let range = OrderedFloat(rect.x)..OrderedFloat(rect.x + rect.width);

        let mut items = vec![];
        for (_, bucket) in self.0.range(range) {
            let range = OrderedFloat(rect.y)..OrderedFloat(rect.y + rect.height);

            let item_list = bucket
                .range(range)
                .map(|(_key, (series_id, item_id))| (series_id.clone(), item_id.clone()));

            items.extend(item_list);
        }

        items
    }
}

impl<SeriesId, ItemId> Default for Items<SeriesId, ItemId> {
    fn default() -> Self {
        Self(BTreeMapFloat::new())
    }
}
