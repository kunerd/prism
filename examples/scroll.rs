use std::ops::RangeInclusive;

use prism::chart::{
    Chart, Labels,
    series::{line_series, point_series},
};

use iced::{
    Element, Length, Task, Theme,
    widget::{Container, column, container, row, text},
};

fn main() -> Result<(), iced::Error> {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .antialiasing(true)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    OnMove(Option<iced::Point>),
    MouseDown(Option<iced::Point>),
    MouseUp(Option<iced::Point>),
}

#[derive(Debug)]
struct App {
    x_offset: f32,
    x_range: RangeInclusive<f32>,
    data: Vec<(f32, f32)>,
    data_1: Vec<Entry>,
    dragging: Dragging,
}

#[derive(Debug)]
struct Entry {
    x: f32,
    y: f32,
}

#[derive(Debug, Default)]
enum Dragging {
    CouldStillBeClick(iced::Point),
    ForSure(iced::Point),
    #[default]
    None,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let data: Vec<_> = (-50..=50)
            .map(|x| x as f32 / 50.0)
            .map(|x| (x, x * x))
            .collect();

        let data_1 = data
            .iter()
            .copied()
            .map(|(x, y)| Entry { x, y: y * 2.0 })
            .collect();

        (
            Self {
                data,
                data_1,
                x_range: -4.0..=4.0,
                x_offset: 0.0,
                dragging: Dragging::None,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "pliced".to_string()
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        let mut update_center = |prev_pos: iced::Point, pos: iced::Point| {
            let shift_x = prev_pos.x - pos.x;

            let new_start = self.x_range.start() + shift_x;
            let new_end = self.x_range.end() + shift_x;

            self.x_range = new_start..=new_end;
            self.x_offset += shift_x;
        };
        match msg {
            Message::MouseDown(pos) => {
                let Dragging::None = self.dragging else {
                    return Task::none();
                };

                if let Some(pos) = pos {
                    self.dragging = Dragging::CouldStillBeClick(pos);
                }
            }
            Message::OnMove(pos) => {
                let Some(pos) = pos else {
                    dbg!("no pos: {:?}", &msg);
                    return Task::none();
                };

                match self.dragging {
                    Dragging::CouldStillBeClick(prev_pos) => {
                        if prev_pos == pos {
                            return Task::none();
                        } else {
                            update_center(prev_pos, pos);
                            self.dragging = Dragging::ForSure(pos);
                        }
                    }
                    Dragging::ForSure(prev_pos) => {
                        update_center(prev_pos, pos);
                        self.dragging = Dragging::ForSure(pos);
                    }
                    Dragging::None => {}
                }
            }
            Message::MouseUp(pos) => {
                let Some(pos) = pos else {
                    dbg!("no pos: {:?}", &msg);
                    return Task::none();
                };
                match self.dragging {
                    Dragging::CouldStillBeClick(_point) => {
                        self.dragging = Dragging::None;
                    }
                    Dragging::ForSure(prev_pos) => {
                        update_center(prev_pos, pos);
                        self.dragging = Dragging::None;
                    }
                    Dragging::None => {}
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let palette = self.theme().palette();

        let top = bound("Top").center_x(Length::Fill);
        let bottom = bound("Bottom").center_x(Length::Fill);
        let left = bound("Left").center_y(Length::Fill);
        let right = bound("Right").center_y(Length::Fill);

        let chart = Chart::<_, (), _>::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .x_range(self.x_range.clone())
            .x_labels(Labels::default().format(&|v| format!("{v:.2}")))
            .y_labels(Labels::default().format(&|v| format!("{v:.5}")))
            .y_range(-2.0..=2.0)
            .push_series(line_series(self.data.iter().copied()).color(palette.primary)) // .push_series(
            .push_series(line_series(&self.data_1).color(palette.success))
            .push_series(
                point_series(self.data.iter().copied().map(|(x, y)| (x, y * 1.5)))
                    .x(&|item| item.0)
                    .y(&|item| item.1)
                    .color(palette.danger),
            )
            .on_press(|state| Message::MouseDown(state.get_offset()))
            .on_release(|state| Message::MouseUp(state.get_offset()))
            .on_move(|state| Message::OnMove(state.get_offset()));

        column![top, row![left, chart, right], bottom].into()
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}

impl From<&Entry> for (f32, f32) {
    fn from(entry: &Entry) -> Self {
        (entry.x, entry.y)
    }
}

fn bound<Message>(label: &str) -> Container<Message> {
    container(text(label).center())
        .padding(10)
        .style(container::bordered_box)
}
