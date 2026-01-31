use iced::{Application, Command, Element, Theme};
use iced::widget::{column, container, text};
use super::{Message, AppState};

pub struct AddrslipsApp {
    state: AppState,
}

impl Application for AddrslipsApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                state: AppState::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Addrslips - Campaign Canvassing Tool".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::None => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            text("Addrslips").size(32),
            text("Campaign Canvassing Address Management"),
            text("GUI coming soon!"),
        ]
        .spacing(20)
        .padding(20);

        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
