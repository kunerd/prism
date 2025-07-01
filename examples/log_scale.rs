use prism::{
    Labels,
    chart::{Axis, Chart, axis, series::line_series},
};

use iced::{Element, Length, Task, Theme, widget::container};

fn main() -> Result<(), iced::Error> {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .antialiasing(true)
        .run()
}

#[derive(Debug, Clone)]
enum Message {}

#[derive(Debug)]
struct App {
    data: Vec<(f32, f32)>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let data = vec![
            (20.0, 10.0),
            (50.0, 20.0),
            (100.0, 50.0),
            (1000.0, 30.0),
            (10_000.0, 50.0),
            (20_000.0, 20.0),
        ];

        (Self { data }, Task::none())
    }

    pub fn title(&self) -> String {
        "pliced".to_string()
    }

    pub fn update(&mut self, _msg: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let palette = self.theme().palette();
        container(
            Chart::<_, ()>::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .x_axis(
                    Axis::new(axis::Alignment::Horizontal)
                        .scale(axis::Scale::Log)
                        .x_tick_marks(
                            [0, 20, 50, 100, 1000, 10_000, 20_000]
                                .into_iter()
                                .map(|v| v as f32)
                                .collect(),
                        ),
                )
                .x_range(20.0..=22_500.0)
                .x_labels(Labels::default().format(&|v| format!("{:.0}", v)))
                .y_range(0.0..=50.0)
                .push_series(line_series(self.data.clone()).color(palette.text)),
        )
        .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
