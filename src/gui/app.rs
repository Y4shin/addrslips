use crate::gui::screens::{Screen, ScreenData, ScreenMessage};
use iced::{Element, Task, Theme, application};

use super::{AppState, Message};

pub struct AddrslipsApp {
    state: AppState,
    screen: ScreenData,
}
/*type Executor = iced::executor::Default;
type Message = Message;
type Theme = Theme;
type Flags = ();*/

impl AddrslipsApp {
    pub fn title(&self) -> String {
        "Addrslips - Campaign Canvassing Tool".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.screen
            .update(message, &mut self.state)
            .map(|msg| match msg {
                ScreenMessage::ScreenMessage(msg) => msg,
                ScreenMessage::ParentMessage(_) => unreachable!(),
            })
    }

    pub fn view(&self) -> Element<Message> {
        self.screen.view().map(|msg| match msg {
            ScreenMessage::ScreenMessage(msg) => msg,
            ScreenMessage::ParentMessage(_) => unreachable!(), // Handle parent messages if needed
        })
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl Default for AddrslipsApp {
    fn default() -> Self {
        Self {
            state: AppState::default(),
            screen: ScreenData::LandingPage(super::screens::landing_page::LandingPageScreen),
        }
    }
}
