use std::convert::Infallible;

use iced::{
    Element, Task,
    widget::{container, text},
};

use crate::gui::{
    AppState,
    screens::{Screen, ScreenMessage},
};

#[derive(Debug, Clone)]
pub struct LoadingPageScreen;


impl Screen for LoadingPageScreen {
    type Message = Infallible;
    type ParentMessage = Infallible;

    fn view(&self) -> Element<'_, ScreenMessage<Self>> {
        container(text("Loading..."))
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .into()
    }

    fn update(
        &mut self,
        _message: Self::Message,
        _state: &mut AppState,
    ) -> Task<ScreenMessage<Self>> {
        // Placeholder update
        Task::none()
    }
}
