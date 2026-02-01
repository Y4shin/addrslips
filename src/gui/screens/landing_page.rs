use std::path::PathBuf;

use iced::{
    Alignment::Center,
    Element, Task,
    widget::{button, column, container, row, text},
};
use rfd::AsyncFileDialog;

use crate::gui::{
    AppState,
    screens::{Screen, ScreenMessage},
};

#[derive(Debug, Clone)]
pub struct LandingPageScreen;

#[derive(Debug, Clone)]
pub enum LandingPageMessage {
    OpenProject,
    CreateProject,
    None,
}

#[derive(Debug, Clone)]
pub enum ParentMessage {
    OpenedProject(PathBuf),
}

impl Screen for LandingPageScreen {
    type Message = LandingPageMessage;
    type ParentMessage = ParentMessage;

    fn view(&self) -> Element<'_, ScreenMessage<Self>> {
        let content = column![
            text("Addrslips").size(32),
            text("Campaign Canvassing Address Management"),
            row![
                button("Open Project").on_press(ScreenMessage::ScreenMessage(
                    LandingPageMessage::OpenProject
                )),
                button("Create Project").on_press(ScreenMessage::ScreenMessage(
                    LandingPageMessage::CreateProject
                )),
            ]
            .spacing(20),
        ]
        .spacing(20)
        .padding(20)
        .align_x(Center);

        container(content)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .into()
    }

    fn update(
        &mut self,
        message: Self::Message,
        _state: &mut AppState,
    ) -> Task<ScreenMessage<Self>> {
        match message {
            LandingPageMessage::OpenProject => Task::perform(
                AsyncFileDialog::new()
                    .add_filter("AddSlips Project", &["asl"])
                    .pick_file(),
                |handle| match handle {
                    Some(data) => ScreenMessage::ParentMessage(ParentMessage::OpenedProject(
                        data.path().to_path_buf(),
                    )),
                    None => ScreenMessage::ScreenMessage(LandingPageMessage::None),
                },
            ),
            LandingPageMessage::CreateProject => Task::perform(
                AsyncFileDialog::new()
                    .set_title("Create New AddSlips Project")
                    .add_filter("AddSlips Project", &["asl"])
                    .save_file(),
                |handle| match handle {
                    Some(data) => ScreenMessage::ParentMessage(ParentMessage::OpenedProject(
                        data.path().to_path_buf(),
                    )),
                    None => ScreenMessage::ScreenMessage(LandingPageMessage::None),
                },
            ),
            LandingPageMessage::None => Task::none(),
        }
    }
}
