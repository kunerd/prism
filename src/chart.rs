mod axis;
mod cartesian;
mod items;
pub mod series;

use axis::Axis;
pub use axis::Labels;
use axis::Tick;
use items::Items;

use core::f32;

use cartesian::Plane;
use iced::Point;
use iced::advanced::Renderer as _;
use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::graphics::text::Paragraph;
use iced::advanced::text::Paragraph as _;
use iced::advanced::widget::{Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, renderer};
use iced::mouse::ScrollDelta;
use iced::widget::canvas::{self, Path, Stroke};
use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::{Element, Length, Rectangle, Size, mouse::Cursor};
use iced::{Font, Renderer, Vector, alignment, touch};

use std::marker::PhantomData;
use std::ops::RangeInclusive;

type StateFn<'a, Message, Id> = Box<dyn Fn(&State<Id>) -> Message + 'a>;

pub struct Chart<'a, Message, Id, Theme = iced::Theme>
where
    Message: Clone,
    Id: Clone,
{
    width: Length,
    height: Length,
    shaping: Shaping,

    margin: Margin,

    x_axis: Axis,
    y_axis: Axis,

    x_ticks: Tick,
    y_ticks: Tick,

    x_labels: Labels<'a>,
    y_labels: Labels<'a>,

    x_range: Option<RangeInclusive<f32>>,
    y_range: Option<RangeInclusive<f32>>,

    x_offset: f32,

    items: Items<Id, usize>,

    series: Vec<Box<dyn series::Series<Id> + 'a>>,
    cache: canvas::Cache,

    on_move: Option<StateFn<'a, Message, Id>>,
    on_press: Option<StateFn<'a, Message, Id>>,
    on_release: Option<StateFn<'a, Message, Id>>,
    on_scroll: Option<StateFn<'a, Message, Id>>,
    //on_right_press: Option<Message>,
    //on_right_release: Option<Message>,
    //on_middle_press: Option<Message>,
    //on_middle_release: Option<Message>,
    //on_enter: Option<Message>,
    //on_move: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    //on_exit: Option<Message>,
    //interaction: Option<mouse::Interaction>,
    theme_: PhantomData<Theme>,
}

impl<'a, Message, Id, Theme> Chart<'a, Message, Id, Theme>
where
    Message: Clone,
    Id: Clone,
{
    const X_RANGE_DEFAULT: RangeInclusive<f32> = 0.0..=10.0;
    const Y_RANGE_DEFAULT: RangeInclusive<f32> = 0.0..=10.0;

    pub fn new() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            shaping: Shaping::default(),

            margin: Margin::default(),

            x_axis: Axis::default(),
            y_axis: Axis::default(),

            x_ticks: Tick::default(),
            y_ticks: Tick::default(),

            x_labels: Labels::default(),
            y_labels: Labels::default(),

            x_range: None,
            y_range: None,

            x_offset: 0.0,

            items: Items::default(),

            series: Vec::new(),
            cache: canvas::Cache::new(),
            on_move: None,
            on_press: None,
            on_release: None,
            on_scroll: None,
            theme_: PhantomData,
        }
    }

    /// set width
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// set height
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// set text shaping
    pub fn text_shaping(mut self, shaping: Shaping) -> Self {
        self.shaping = shaping;
        self
    }

    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    pub fn x_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.x_range = Some(range);
        self
    }

    pub fn y_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.y_range = Some(range);
        self
    }

    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = axis;
        self
    }

    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = axis;
        self
    }

    pub fn x_offset(mut self, offset: f32) -> Self {
        self.x_offset = offset;
        self
    }

    pub fn x_ticks(mut self, ticks: Tick) -> Self {
        self.x_ticks = ticks;
        self
    }

    pub fn y_ticks(mut self, ticks: Tick) -> Self {
        self.y_ticks = ticks;
        self
    }

    pub fn x_labels(mut self, labels: Labels<'a>) -> Self {
        self.x_labels = labels;
        self
    }

    pub fn y_labels(mut self, labels: Labels<'a>) -> Self {
        self.y_labels = labels;
        self
    }

    pub fn push_series(mut self, series: impl series::Series<Id> + 'a) -> Self {
        if let Some((id, items)) = series.items() {
            self.items.add_series(id, &items);
        }
        self.series.push(Box::new(series));

        self
    }

    pub fn extend_series(
        self,
        series_list: impl IntoIterator<Item = impl series::Series<Id> + 'a>,
    ) -> Self {
        series_list.into_iter().fold(self, Self::push_series)
    }

    pub fn on_press(mut self, msg: impl Fn(&State<Id>) -> Message + 'a) -> Self {
        self.on_press = Some(Box::new(msg));
        self
    }

    pub fn on_release(mut self, msg: impl Fn(&State<Id>) -> Message + 'a) -> Self {
        self.on_release = Some(Box::new(msg));
        self
    }

    pub fn on_move(mut self, msg: impl Fn(&State<Id>) -> Message + 'a) -> Self {
        self.on_move = Some(Box::new(msg));
        self
    }

    pub fn on_scroll(mut self, msg: impl Fn(&State<Id>) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(msg));
        self
    }

    fn draw_x_axis(&self, frame: &mut canvas::Frame, plane: &Plane) {
        let bounds = frame.size();

        let mut scaled_bottom_left = plane.scale_to_cartesian(plane.bottom_left());
        let mut scaled_bottom_right = plane.scale_to_cartesian(plane.bottom_right());

        let label_height = 10.0;
        if scaled_bottom_left.x > bounds.height - label_height {
            // TODO minus label height
            scaled_bottom_left.x = bounds.width - label_height;
            scaled_bottom_right.x = bounds.width - label_height;
        }

        frame.stroke(
            &Path::line(scaled_bottom_left, scaled_bottom_right),
            Stroke::default()
                .with_width(self.x_axis.width)
                .with_color(self.x_axis.color),
        );

        // ticks
        let tick_width = plane.x.length / self.x_ticks.amount as f32;
        let mut draw_x_tick = |x| {
            let x_scaled = plane.scale_to_cartesian_x(x);
            let y_scaled = plane.scale_to_cartesian_y(0.0);

            let half_tick_height = self.x_ticks.height / 2.0;
            let x_start = Point {
                x: x_scaled,
                y: y_scaled - half_tick_height,
            };
            let x_end = Point {
                x: x_scaled,
                y: y_scaled + half_tick_height,
            };

            frame.stroke(
                &Path::line(x_start, x_end),
                Stroke::default()
                    .with_width(self.x_ticks.width)
                    .with_color(self.x_ticks.color),
            );

            let label = self
                .x_labels
                .format
                .map_or_else(|| format!("{x}"), |fmt| fmt(&x));

            frame.fill_text(canvas::Text {
                content: label,
                size: self.x_labels.font_size.unwrap_or(12.into()),
                position: Point {
                    x: x_scaled,
                    // TODO remove magic number,
                    y: y_scaled + 8.0,
                },
                // TODO use theme
                color: self.x_labels.color.unwrap_or(iced::Color::WHITE),
                // TODO edge case center tick
                align_x: alignment::Horizontal::Center,
                align_y: alignment::Vertical::Top,
                font: Font::MONOSPACE,
                ..canvas::Text::default()
            });
        };

        let left = (plane.x.min / tick_width).ceil() as i32;
        for i in left..0 {
            draw_x_tick(i as f32 * tick_width);
        }

        let right = (plane.x.max / tick_width).floor() as i32;
        for i in 0..=right {
            draw_x_tick(i as f32 * tick_width);
        }
    }

    fn draw_y_axis(&self, frame: &mut canvas::Frame, plane: &Plane) {
        let text_width = |text: &str, font_size| {
            let text = iced::advanced::text::Text {
                content: text,
                size: font_size,
                line_height: LineHeight::default(),
                bounds: iced::Size::INFINITY,
                font: Font::MONOSPACE,
                align_x: iced::advanced::text::Alignment::Right,
                align_y: alignment::Vertical::Center,
                shaping: Shaping::Basic,
                wrapping: Wrapping::default(),
            };

            let paragraph = Paragraph::with_text(text);
            paragraph.min_bounds().width
        };

        let mut max_label_width = 0.0f32;
        let tick_width = plane.y.length / self.y_ticks.amount as f32;
        let down = (plane.y.min / tick_width).ceil() as i32;
        for i in down..0 {
            let y = plane.scale_to_cartesian_y(i as f32);
            let label = self
                .y_labels
                .format
                .map_or_else(|| format!("{y}"), |fmt| fmt(&y));
            let font_size = self.y_labels.font_size.unwrap_or(12.into());
            let label_width = text_width(&label, font_size);
            max_label_width = max_label_width.max(label_width);
        }

        let up = (plane.y.max / tick_width).floor() as i32;
        for i in 1..=up {
            let y = plane.scale_to_cartesian_y(i as f32);
            let label = self
                .y_labels
                .format
                .map_or_else(|| format!("{y}"), |fmt| fmt(&y));
            let font_size = self.y_labels.font_size.unwrap_or(12.into());
            let label_width = text_width(&label, font_size);
            max_label_width = max_label_width.max(label_width);
        }
        let bounds = frame.size();

        let mut scaled_bottom_center = plane.scale_to_cartesian(plane.bottom_center());
        let mut scaled_top_center = plane.scale_to_cartesian(plane.top_center());

        if scaled_bottom_center.x - max_label_width <= 0.0 {
            // TODO minus label height
            scaled_bottom_center.x = max_label_width;
            scaled_top_center.x = max_label_width;
        } else if scaled_bottom_center.x > bounds.width {
            scaled_bottom_center.x = bounds.width;
            scaled_top_center.x = bounds.width;
        }

        frame.stroke(
            &Path::line(scaled_bottom_center, scaled_top_center),
            Stroke::default()
                .with_width(self.y_axis.width)
                .with_color(self.y_axis.color),
        );

        let mut draw_y_tick = |y| {
            let x_scaled = scaled_top_center.x;
            let y_scaled = plane.scale_to_cartesian_y(y);

            let half_tick_height = self.y_ticks.height / 2.0;
            let start = Point {
                x: x_scaled - half_tick_height,
                y: y_scaled,
            };
            let end = Point {
                x: x_scaled + half_tick_height,
                y: y_scaled,
            };

            frame.stroke(
                &Path::line(start, end), //.transform(&Transform2D::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0)),
                Stroke::default()
                    .with_width(self.y_ticks.width)
                    .with_color(self.y_ticks.color),
            );

            let label = self
                .y_labels
                .format
                .map_or_else(|| format!("{y}"), |fmt| fmt(&y));

            let font_size = self.y_labels.font_size.unwrap_or(12.into());
            frame.fill_text(canvas::Text {
                content: label.clone(),
                size: font_size,
                position: Point {
                    // TODO remove magic number,
                    x: x_scaled - 8.0,
                    y: y_scaled,
                },
                // TODO use theme
                color: self.y_labels.color.unwrap_or(iced::Color::WHITE),
                // TODO edge case center tick
                align_x: alignment::Horizontal::Right,
                align_y: alignment::Vertical::Center,
                font: Font::MONOSPACE,
                ..canvas::Text::default()
            });
        };

        for i in down..0 {
            draw_y_tick(i as f32 * tick_width);
        }

        let up = (plane.y.max / tick_width).floor() as i32;
        for i in 1..=up {
            draw_y_tick(i as f32 * tick_width);
        }
    }

    fn draw_data(&self, frame: &mut canvas::Frame, plane: &Plane) {
        for series in &self.series {
            series.draw(frame, plane);
        }
    }

    fn compute_x_range_from_series(&self) -> RangeInclusive<f32> {
        let mut max: Option<RangeInclusive<f32>> = None;

        for series in &self.series {
            let cur = series.x_range();

            max = match max {
                Some(max) => {
                    let min = max.start().min(*cur.start());
                    let max = max.end().max(*cur.end());

                    Some(min..=max)
                }
                None => Some(cur),
            }
        }

        max.unwrap_or(Self::X_RANGE_DEFAULT)
    }

    fn compute_y_range_from_series(&self) -> RangeInclusive<f32> {
        let mut max: Option<RangeInclusive<f32>> = None;

        for series in &self.series {
            let cur = series.y_range();

            max = match max {
                Some(max) => {
                    let min = max.start().min(*cur.start());
                    let max = max.end().max(*cur.end());

                    Some(min..=max)
                }
                None => Some(cur),
            }
        }

        max.unwrap_or(Self::Y_RANGE_DEFAULT)
    }
}

impl<Message, Id, Theme> Widget<Message, Theme, Renderer> for Chart<'_, Message, Id, Theme>
where
    Message: Clone,
    Id: 'static + Clone + PartialEq,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Id>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Id>::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![]
    }

    #[inline]
    fn layout(
        &self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let node = layout::atomic(limits, self.width, self.height);
        //limits.resolve(self.width, self.height, Size::ZERO);

        let x_range = match &self.x_range {
            Some(range) => range,
            None => &self.compute_x_range_from_series(),
        };

        let y_range = match &self.y_range {
            Some(range) => range,
            None => &self.compute_y_range_from_series(),
        };

        //let node = layout::Node::new(size);
        let bounds = node.bounds();

        let x_margin_min = self.margin.left;
        let x_margin_max = self.margin.right;
        let y_margin_min = self.margin.bottom;
        let y_margin_max = self.margin.top;

        // let x_range = &(x_range.start() + self.x_offset..=x_range.end() + self.x_offset);

        let plane = Plane {
            x: cartesian::Axis::new(x_range, x_margin_min, x_margin_max, bounds.width),
            y: cartesian::Axis::new(y_range, y_margin_min, y_margin_max, bounds.height),
        };

        let state = tree.state.downcast_mut::<State<Id>>();
        state.plane = Some(plane);

        node
    }

    #[inline]
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let state: &State<Id> = tree.state.downcast_ref();
        let Some(plane) = &state.plane else {
            return;
        };

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_data(frame, plane);
            self.draw_x_axis(frame, plane);
            self.draw_y_axis(frame, plane);
        });

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            renderer.draw_geometry(geometry)
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let Some(cursor_position) = cursor.position() else {
            return;
        };

        let bounds = layout.bounds();

        let state: &mut State<Id> = tree.state.downcast_mut();
        let relative_position = cursor_position - Vector::new(bounds.x, bounds.y);
        state.prev_position = state.cursor_position;
        state.cursor_position = Some(relative_position);

        //if state.cursor_position != cursor_position || state.bounds != bounds {
        if bounds.contains(cursor_position) {
            if let Some(message) = self.on_press.as_ref() {
                if let iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | iced::Event::Touch(touch::Event::FingerPressed { .. }) = event
                {
                    shell.publish(message(state));

                    return;
                }
            }

            if let Some(message) = self.on_release.as_ref() {
                if let iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | iced::Event::Touch(touch::Event::FingerLifted { .. }) = event
                {
                    shell.publish(message(state));

                    return;
                }
            }

            if let Some(message) = self.on_move.as_ref() {
                if let iced::Event::Mouse(mouse::Event::CursorMoved { .. })
                | iced::Event::Touch(touch::Event::FingerMoved { .. }) = event
                {
                    if let (Some(coords), Some(plane)) = (state.get_coords(), &state.plane) {
                        let iter = self
                            .series
                            .iter()
                            .filter_map(|s| s.id().map(|id| (id, s.collision_box().unwrap())));

                        let mut item_list = vec![];
                        for (series_id, collision_box) in iter {
                            let top_left = Point::new(
                                coords.x - collision_box.width / 2.0 / plane.x.scale,
                                coords.y - collision_box.height / 2.0 / plane.y.scale,
                            );
                            let rect = Rectangle::new(
                                top_left,
                                Size::new(
                                    collision_box.width / plane.x.scale,
                                    collision_box.height / plane.y.scale,
                                ),
                            );
                            item_list.extend(
                                self.items
                                    .collision(rect)
                                    .into_iter()
                                    .filter(|i| i.0 == series_id),
                            );
                        }
                        state.item_list = Some(item_list);
                    }

                    shell.publish(message(state));

                    return;
                }
            }

            if let Some(message) = self.on_scroll.as_ref() {
                if let iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
                    state.scroll_delta = Some(*delta);

                    shell.publish(message(state));
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        _layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::None
    }
}

impl<Message, Id, Theme> Default for Chart<'_, Message, Id, Theme>
where
    Message: Clone,
    Id: Clone,
{
    fn default() -> Self {
        Chart::new()
    }
}

/// Local state of the [`Chart`].
pub struct State<Id>
where
    Id: Clone,
{
    plane: Option<Plane>,
    prev_position: Option<Point>,
    cursor_position: Option<Point>,
    scroll_delta: Option<ScrollDelta>,
    item_list: Option<Vec<(Id, usize)>>,
}

impl<Id> State<Id>
where
    Id: Clone,
{
    pub fn get_cursor_position(&self) -> Option<Point> {
        self.cursor_position
    }

    fn get_cartesian(&self, point: Point) -> Option<Point> {
        self.plane.as_ref().map(|p| p.get_cartesian(point))
    }

    pub fn get_coords(&self) -> Option<Point> {
        self.get_cartesian(self.cursor_position?)
    }

    pub fn get_offset(&self) -> Option<Point> {
        let pos = self.cursor_position?;

        self.plane.as_ref().map(|p| p.get_offset(pos))
    }

    pub fn x_range(&self) -> Option<RangeInclusive<f32>> {
        let plane = self.plane.as_ref()?;

        let min = plane.x.min;
        let max = plane.x.max;

        Some(min..=max)
    }

    pub fn scroll_delta(&self) -> Option<ScrollDelta> {
        self.scroll_delta
    }

    pub fn items(&self) -> Option<&Vec<(Id, usize)>> {
        self.item_list.as_ref()
    }
}

impl<Id> Default for State<Id>
where
    Id: Clone,
{
    fn default() -> Self {
        Self {
            plane: Default::default(),
            prev_position: Default::default(),
            cursor_position: Default::default(),
            scroll_delta: Default::default(),
            item_list: Default::default(),
        }
    }
}

impl<'a, Message, Id, Theme> From<Chart<'a, Message, Id, Theme>> for Element<'a, Message, Theme>
where
    Message: 'a + Clone,
    Theme: 'a,
    Id: 'static + Clone + PartialEq,
{
    fn from(chart: Chart<'a, Message, Id, Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(chart)
    }
}

pub struct Margin {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Margin {
    const MARGIN_DEFAULT: f32 = 0.0;
}

impl Default for Margin {
    fn default() -> Self {
        Self {
            top: Self::MARGIN_DEFAULT,
            bottom: Self::MARGIN_DEFAULT,
            left: Self::MARGIN_DEFAULT,
            right: Self::MARGIN_DEFAULT,
        }
    }
}
