use prism::chart::{
    Chart,
    series::{line_series, point, point_series},
};

use iced::{Element, Length, Task, Theme, widget::container};

fn main() -> Result<(), iced::Error> {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .antialiasing(true)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    OnMove(Option<Vec<(ItemId, usize)>>),
}

#[derive(Debug)]
struct App {
    data: Vec<StyledPoint>,
    selected_item: Option<usize>,
}

#[derive(Debug)]
struct StyledPoint {
    coords: (f32, f32),
    color: iced::Color,
    border_color: iced::Color,
    border: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ItemId {
    PointList,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let red = iced::Color::from_rgb8(255, 0, 0);
        let green = iced::Color::from_rgb8(0, 255, 0);
        let blue = iced::Color::from_rgb8(42, 142, 255);
        let yellow = iced::Color::from_rgb8(238, 230, 0);

        let border = 2.0;

        let data = vec![
            StyledPoint {
                coords: (0.0, 0.0),
                color: yellow,
                border_color: red,
                border,
            },
            StyledPoint {
                coords: (1.0, 1.0),
                color: yellow,
                border_color: green,
                border,
            },
            StyledPoint {
                coords: (2.0, 1.0),
                color: green,
                border_color: blue,
                border: 1.0,
            },
            StyledPoint {
                coords: (3.0, 0.0),
                color: green,
                border_color: yellow,
                border: 3.0,
            },
        ];

        (
            Self {
                data,
                selected_item: None,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "pliced".to_string()
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        if let Message::OnMove(Some(items)) = &msg {
            self.selected_item = items.first().map(|(_, index)| *index)
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let palette = self.theme().palette();
        container(
            Chart::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .x_range(-0.5..=3.5)
                .y_range(-0.5..=1.5)
                .push_series(line_series(&self.data).color(palette.text))
                .push_series(
                    point_series(self.data.iter())
                        .color(palette.danger)
                        .style_for_each(|_index, item| point::Style {
                            color: Some(item.color),
                            border_color: Some(item.border_color),
                            border: item.border,
                            ..Default::default()
                        })
                        .with_id(ItemId::PointList),
                )
                .on_move(|state| Message::OnMove(state.items().cloned())),
        )
        .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}

impl From<&StyledPoint> for (f32, f32) {
    fn from(point: &StyledPoint) -> Self {
        let (x, y) = point.coords;
        (x, y)
    }
}

impl From<StyledPoint> for (f32, f32) {
    fn from(point: StyledPoint) -> Self {
        let (x, y) = point.coords;
        (x, y)
    }
}
