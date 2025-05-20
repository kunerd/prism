use core::f32;
use std::marker::PhantomData;
use std::ops::Range;

use crate::backend::{self, IcedChartBackend};
use crate::cartesian::Cartesian;
use crate::event::{self};
use crate::program::Program;

use iced::advanced::graphics::geometry;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{layout, mouse, renderer, Clipboard, Layout, Shell, Widget};
use iced::widget::canvas;
use iced::widget::text::Shaping;
use iced::{mouse::Cursor, Element, Length, Rectangle, Size};
use iced::{touch, Point, Renderer, Vector};
use plotters::coord::types::RangedCoordf32;
use plotters::prelude::*;
use plotters::style::Color as _;
use plotters_backend::text_anchor::Pos;
use plotters_backend::BackendColor;

pub type ChartBuilderFn<Renderer = iced::Renderer> =
    Box<dyn for<'a, 'b> Fn(&mut ChartBuilder<'a, 'b, IcedChartBackend<'b, Renderer>>)>;

pub struct Chart<
    'a,
    Message,
    P = Attributes<'a, Message>,
    Theme = iced::Theme,
    Renderer = iced::Renderer,
> where
    Message: Clone,
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    program: P,
    width: Length,
    height: Length,
    shaping: Shaping,
    on_press: Option<Message>,
    //on_release: Option<Message>,
    //on_right_press: Option<Message>,
    //on_right_release: Option<Message>,
    //on_middle_press: Option<Message>,
    //on_middle_release: Option<Message>,
    //on_enter: Option<Message>,
    //on_move: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    //on_exit: Option<Message>,
    //interaction: Option<mouse::Interaction>,
    cache: Option<&'a geometry::Cache<Renderer>>,
    theme_: PhantomData<Theme>,
    renderer_: PhantomData<Renderer>,
}

impl<'a, Message, Theme, Renderer> Chart<'_, Message, Attributes<'a, Message>, Theme, Renderer>
where
    Message: Clone,
    Renderer: geometry::Renderer,
    Attributes<'a, Message>: Program<Message, Theme, Renderer>,
{
    pub fn new() -> Self {
        let program = Attributes::default();

        Self::from_program(program)
    }

    pub fn x_range(mut self, range: Range<f32>) -> Self {
        self.program.x_range = AxisRange::Custom(range);

        self
    }

    pub fn y_range(mut self, range: Range<f32>) -> Self {
        self.program.y_range = AxisRange::Custom(range);

        self
    }

    pub fn push_series(mut self, series: impl Into<Series>) -> Self {
        let series = series.into();

        if let AxisRange::Automatic(x_range) = self.program.x_range {
            let x_min_cur = x_range.as_ref().map_or(f32::INFINITY, |range| range.start);
            let x_max_cur = x_range
                .as_ref()
                .map_or(f32::NEG_INFINITY, |range| range.end);

            let (x_min, x_max) = {
                let iter = match &series {
                    Series::Line(line_series) => line_series.data.iter(),
                    Series::Point(point_series) => point_series.data.iter(),
                };

                iter.fold((x_min_cur, x_max_cur), |(x_min, x_max), (cur_x, _)| {
                    (x_min.min(*cur_x), x_max.max(*cur_x))
                })
            };

            self.program.x_range = AxisRange::Automatic(Some(x_min..x_max));
        }

        if let AxisRange::Automatic(y_range) = self.program.y_range {
            let y_min_cur = y_range.as_ref().map_or(f32::INFINITY, |range| range.start);
            let y_max_cur = y_range
                .as_ref()
                .map_or(f32::NEG_INFINITY, |range| range.end);

            let (y_min, y_max) = {
                let iter = match &series {
                    Series::Line(line_series) => line_series.data.iter(),
                    Series::Point(point_series) => point_series.data.iter(),
                };

                iter.fold((y_min_cur, y_max_cur), |(y_min, y_max), (_, cur_y)| {
                    (y_min.min(*cur_y), y_max.max(*cur_y))
                })
            };

            self.program.y_range = AxisRange::Automatic(Some(y_min..y_max));
        }

        self.program.series.push(series);

        self
    }

    pub fn extend_series(
        self,
        series_list: impl IntoIterator<Item = impl Into<Series>> + Clone,
    ) -> Self {
        series_list.into_iter().fold(self, Self::push_series)
    }

    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }

    pub fn on_move(mut self, msg: impl Fn(iced::Point, Cartesian) -> Message + 'a) -> Self {
        self.program.on_move = Some(Box::new(msg));
        self
    }

    pub fn on_scroll(
        mut self,
        msg: impl Fn(iced::Point, mouse::ScrollDelta, Cartesian) -> Message + 'a,
    ) -> Self {
        self.program.on_scroll = Some(Box::new(msg));
        self
    }
}

impl<'a, Message, P, Theme, Renderer> Chart<'a, Message, P, Theme, Renderer>
where
    Message: Clone,
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    pub fn from_program(program: P) -> Self {
        Self {
            program,
            width: Length::Fill,
            height: Length::Fill,
            shaping: Default::default(),
            cache: None,
            on_press: None,
            //on_release: None,
            //on_enter: None,
            //on_move: None,
            //on_exit: None,
            //interaction: None,
            theme_: PhantomData,
            renderer_: PhantomData,
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

    pub fn with_cache(mut self, cache: &'a geometry::Cache<Renderer>) -> Self {
        self.cache = Some(cache);
        self
    }
}

impl<P, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Chart<'_, Message, P, Theme, Renderer>
where
    Message: Clone,
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        struct Tag<T>(T);
        vec![Tree {
            tag: tree::Tag::of::<Tag<P::State>>(),
            state: tree::State::new(P::State::default()),
            children: vec![],
        }]
    }

    #[inline]
    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(self.width, self.height, Size::ZERO);
        layout::Node::new(size)
    }

    #[inline]
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let state = tree.children[0].state.downcast_ref::<P::State>();

        let geometry = if let Some(cache) = &self.cache {
            cache.draw(renderer, bounds.size(), |frame| {
                let root = IcedChartBackend::new(frame, self.shaping).into_drawing_area();
                let mut chart_builder = ChartBuilder::on(&root);

                self.program
                    .draw(state, &mut chart_builder, theme, bounds, cursor);

                root.present().unwrap();
            })
        } else {
            let mut frame = canvas::Frame::new(renderer, bounds.size());
            let root = IcedChartBackend::new(&mut frame, self.shaping).into_drawing_area();
            let mut chart_builder = ChartBuilder::on(&root);

            self.program
                .draw(state, &mut chart_builder, theme, bounds, cursor);

            root.present().unwrap();

            frame.into_geometry()
        };

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            renderer.draw_geometry(geometry);
        });
    }

    #[inline]
    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _rectangle: &Rectangle,
    ) -> event::Status {
        let state: &mut State = tree.state.downcast_mut();

        let cursor_position = cursor.position();
        let bounds = layout.bounds();

        if state.cursor_position != cursor_position || state.bounds != bounds {
            if let Some(message) = self.on_press.as_ref() {
                if let iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | iced::Event::Touch(touch::Event::FingerPressed { .. }) = event
                {
                    shell.publish(message.clone());

                    return event::Status::Captured;
                }
            }

            let canvas_event = match event {
                iced::Event::Mouse(mouse_event) => Some(event::Event::Mouse(mouse_event)),
                iced::Event::Touch(touch_event) => Some(event::Event::Touch(touch_event)),
                iced::Event::Keyboard(keyboard_event) => {
                    Some(event::Event::Keyboard(keyboard_event))
                }
                iced::Event::Window(_) => None,
            };

            if let Some(canvas_event) = canvas_event {
                let state = tree.children[0].state.downcast_mut::<P::State>();

                let (event_status, message) =
                    self.program.update(state, canvas_event, bounds, cursor);

                if let Some(message) = message {
                    shell.publish(message);
                }

                return event_status;
            }
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let state = tree.children[0].state.downcast_ref::<P::State>();

        self.program.mouse_interaction(state, bounds, cursor)
    }
}

/// Local state of the [`Chart`].
#[derive(Default)]
struct State {
    //is_hovered: bool,
    bounds: Rectangle,
    cursor_position: Option<Point>,
}

impl<'a, Message, Theme, Renderer> Default
    for Chart<'a, Message, Attributes<'a, Message>, Theme, Renderer>
where
    Message: Clone,
    Renderer: 'a + geometry::Renderer,
    Attributes<'a, Message>: Program<Message, Theme, Renderer>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, P, Message, Theme, Renderer> From<Chart<'a, Message, P, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + geometry::Renderer,
    P: 'a + Program<Message, Theme, Renderer>,
{
    fn from(
        chart: Chart<'a, Message, P, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(chart)
    }
}

pub struct Attributes<'a, Message>
where
    Message: Clone,
{
    x_range: AxisRange<Range<f32>>,
    y_range: AxisRange<Range<f32>>,
    series: Vec<Series>,

    on_move: Option<Box<dyn Fn(iced::Point, Cartesian) -> Message + 'a>>,
    on_scroll: Option<Box<dyn Fn(iced::Point, mouse::ScrollDelta, Cartesian) -> Message + 'a>>,
}

impl<Message> Default for Attributes<'_, Message>
where
    Message: Clone,
{
    fn default() -> Self {
        Self {
            x_range: Default::default(),
            y_range: Default::default(),
            series: Default::default(),

            on_move: Default::default(),
            on_scroll: Default::default(),
        }
    }
}

#[derive(Clone)]
pub enum AxisRange<T> {
    Custom(T),
    Automatic(Option<T>),
}

impl<T> Default for AxisRange<T> {
    fn default() -> Self {
        Self::Automatic(None)
    }
}

impl<Message> Attributes<'_, Message>
where
    Message: Clone,
{
    const X_RANGE_DEFAULT: Range<f32> = 0.0..10.0;
    const Y_RANGE_DEFAULT: Range<f32> = 0.0..10.0;
}

impl<Message> Program<Message> for Attributes<'_, Message>
where
    Message: Clone,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        chart: &mut ChartBuilder<backend::IcedChartBackend<Renderer>>,
        theme: &iced::Theme,
        _bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) {
        let x_range = match self.x_range.clone() {
            AxisRange::Custom(x_range) => x_range,
            AxisRange::Automatic(Some(x_range)) => x_range,
            AxisRange::Automatic(None) => Attributes::<Message>::X_RANGE_DEFAULT,
        };

        let y_range = match self.y_range.clone() {
            AxisRange::Custom(y_range) => y_range,
            AxisRange::Automatic(Some(y_range)) => y_range,
            AxisRange::Automatic(None) => Attributes::<Message>::Y_RANGE_DEFAULT,
        };

        let mut chart = chart
            .x_label_area_size(10)
            .margin(20)
            .build_cartesian_2d(x_range, y_range)
            .unwrap();

        let text_color = Color(theme.palette().text);
        let label_style = TextStyle {
            font: "sans".into(),
            color: text_color.into(),
            pos: Pos::default(),
        };
        chart
            .configure_mesh()
            //.disable_mesh()
            .label_style(label_style)
            .bold_line_style(GREEN.mix(0.1))
            .light_line_style(BLUE.mix(0.1))
            .draw()
            .unwrap();

        for s in &self.series {
            match s {
                Series::Line(line_series) => {
                    chart
                        .draw_series(plotters::series::LineSeries::from(line_series))
                        .unwrap();
                }
                Series::Point(point_series) => {
                    chart
                        .draw_series(plotters::series::PointSeries::of_element(
                            point_series.data.iter().copied(),
                            5,
                            ShapeStyle::from(&RED).filled(),
                            &|coord, size, style| {
                                EmptyElement::at(coord) + Circle::new((0, 0), size, style)
                            },
                        ))
                        .unwrap();
                }
            }
        }
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: event::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (event::Status, Option<Message>) {
        let x_range = match self.x_range.clone() {
            AxisRange::Custom(x_range) => x_range,
            AxisRange::Automatic(Some(x_range)) => x_range,
            AxisRange::Automatic(None) => Attributes::<Message>::X_RANGE_DEFAULT,
        };

        let y_range = match self.y_range.clone() {
            AxisRange::Custom(y_range) => y_range,
            AxisRange::Automatic(Some(y_range)) => y_range,
            AxisRange::Automatic(None) => Attributes::<Message>::Y_RANGE_DEFAULT,
        };

        let coord_spec: Cartesian2d<RangedCoordf32, RangedCoordf32> = Cartesian2d::new(
            x_range,
            y_range,
            (0..bounds.width as i32, 0..bounds.height as i32),
        );

        if let Some(on_scroll) = self.on_scroll.as_ref() {
            if let event::Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
                let Cursor::Available(position) = cursor else {
                    return (event::Status::Ignored, None);
                };

                let origin = bounds.position();
                let position = position - origin;
                let position = iced::Point::new(position.x, position.y);

                return (
                    event::Status::Captured,
                    Some(on_scroll(position, delta, Cartesian::new(coord_spec))),
                );
            }
        }

        if let Some(on_move) = self.on_move.as_ref() {
            if let event::Event::Mouse(mouse::Event::CursorMoved { position }) = event {
                let origin = bounds.position();
                let position = position - origin;
                let position = iced::Point::new(position.x, position.y);

                return (
                    event::Status::Captured,
                    Some(on_move(position, Cartesian::new(coord_spec))),
                );
            }
        }

        (event::Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        iced::mouse::Interaction::default()
    }
}

#[derive(Clone)]
pub enum Series {
    Line(LineSeries),
    Point(PointSeries),
}

#[derive(Clone)]
pub struct LineSeries {
    pub data: Vec<(f32, f32)>,
    pub color: Color,
}

impl LineSeries {
    pub fn new(iter: impl IntoIterator<Item = (f32, f32)>) -> Self {
        Self {
            data: iter.into_iter().collect(),
            color: Color(iced::Color::BLACK),
        }
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }
}

impl From<LineSeries> for Series {
    fn from(line_series: LineSeries) -> Self {
        Self::Line(line_series)
    }
}

#[derive(Clone)]
pub struct PointSeries {
    pub data: Vec<(f32, f32)>,
    pub color: Color,
}

impl PointSeries {
    pub fn new(iter: impl IntoIterator<Item = (f32, f32)>) -> Self {
        Self {
            data: iter.into_iter().collect(),
            color: Color(iced::Color::BLACK),
        }
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }
}

impl From<PointSeries> for Series {
    fn from(point_series: PointSeries) -> Self {
        Self::Point(point_series)
    }
}

pub fn line_series(iter: impl IntoIterator<Item = (f32, f32)>) -> LineSeries {
    LineSeries::new(iter)
}

pub fn point_series(iter: impl IntoIterator<Item = (f32, f32)>) -> PointSeries {
    PointSeries::new(iter)
}

impl<Backend> From<&LineSeries> for plotters::series::LineSeries<Backend, (f32, f32)>
where
    Backend: plotters::backend::DrawingBackend,
{
    fn from(series: &LineSeries) -> Self {
        let style: ShapeStyle = series.color.into();
        Self::new(series.data.clone(), style)
    }
}

#[derive(Clone, Copy)]
pub struct Color(pub iced::Color);

impl From<iced::Color> for Color {
    fn from(color: iced::Color) -> Self {
        Self(color)
    }
}

impl From<Color> for plotters::style::RGBAColor {
    fn from(color: Color) -> Self {
        let color = color.0.into_rgba8();
        Self(color[0], color[1], color[2], color[3] as f64 / 256.0)
    }
}

impl From<Color> for ShapeStyle {
    fn from(color: Color) -> Self {
        ShapeStyle {
            color: color.into(),
            filled: true,
            stroke_width: 2,
        }
    }
}

impl From<&Color> for ShapeStyle {
    fn from(color: &Color) -> Self {
        ShapeStyle {
            color: (*color).into(),
            filled: true,
            stroke_width: 2,
        }
    }
}

impl From<Color> for BackendColor {
    fn from(color: Color) -> Self {
        let color: plotters::style::RGBAColor = color.into();
        color.to_backend_color()
    }
}
