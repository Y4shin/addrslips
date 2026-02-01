pub mod landing_page;
pub mod loading_page;
pub mod select_area;

use iced::{Element, Task};

use crate::{
    core::db::ProjectDb,
    gui::{AppState, Message, state::ProjectState},
};

#[derive(Debug, Clone)]
pub enum ScreenMessage<S: Screen> {
    ScreenMessage(S::Message),
    ParentMessage(S::ParentMessage),
}

pub trait Screen: Sized {
    type Message: std::fmt::Debug;
    type ParentMessage: std::fmt::Debug;
    fn view(&self) -> Element<'_, ScreenMessage<Self>>;
    fn update(&mut self, message: Self::Message, state: &mut AppState)
    -> Task<ScreenMessage<Self>>;
}

#[derive(Debug, Clone)]
pub enum ScreenData {
    LandingPage(landing_page::LandingPageScreen),
    LoadingPage(loading_page::LoadingPageScreen),
    SelectAreaPage(select_area::SelectAreaScreen),
}

impl Screen for ScreenData {
    type Message = Message;
    type ParentMessage = std::convert::Infallible;
    fn view(&self) -> Element<'_, ScreenMessage<Self>> {
        match self {
            ScreenData::LandingPage(screen) => screen.view().map(Message::LandingPage),
            ScreenData::LoadingPage(screen) => screen.view().map(Message::LoadingPageMessage),
            ScreenData::SelectAreaPage(screen) => screen.view().map(Message::SelectAreaMessage),
        }
        .map(ScreenMessage::ScreenMessage)
    }

    fn update(
        &mut self,
        message: Self::Message,
        state: &mut AppState,
    ) -> Task<ScreenMessage<Self>> {
        match (self, message) {
            (x, Message::ChangeScreen(screen)) => {
                *x = screen;
                Task::none()
            }
            (x, Message::LoadProject(project, area_select_screen)) => {
                state.current_project = Some(project);
                *x = ScreenData::SelectAreaPage(area_select_screen);
                Task::none()
            }
            (ScreenData::LandingPage(page), Message::LandingPage(msg)) => match msg {
                ScreenMessage::ScreenMessage(msg) => page
                    .update(msg, state)
                    .map(Message::LandingPage)
                    .map(ScreenMessage::ScreenMessage),
                ScreenMessage::ParentMessage(parent_msg) => match parent_msg {
                    landing_page::ParentMessage::OpenedProject(path) => {
                        // Handle opening project logic here
                        // For now, just return none
                        assert!(state.current_project.is_none());
                        Task::done(ScreenMessage::ScreenMessage(Message::ChangeScreen(
                            ScreenData::LoadingPage(loading_page::LoadingPageScreen),
                        )))
                        .chain(Task::perform(
                            async {
                                let project = ProjectState {
                                    project_db: ProjectDb::new(path).await?,
                                    area_db: None,
                                };
                                let area_select_screen =
                                    select_area::SelectAreaScreen::new(&project).await;
                                Ok((project, area_select_screen))
                            },
                            |result: Result<
                                (ProjectState<'static>, select_area::SelectAreaScreen),
                                anyhow::Error,
                            >| {
                                let (project, area_select_screen) =
                                    result.expect("Failed to open project");
                                ScreenMessage::ScreenMessage(Message::LoadProject(
                                    project,
                                    area_select_screen,
                                ))
                            },
                        ))
                    }
                },
            },
            (ScreenData::SelectAreaPage(page), Message::SelectAreaMessage(msg)) => match msg {
                ScreenMessage::ScreenMessage(msg) => page
                    .update(msg, state)
                    .map(Message::SelectAreaMessage)
                    .map(ScreenMessage::ScreenMessage),
                ScreenMessage::ParentMessage(_parent_msg) => {
                    // Handle parent messages if needed
                    Task::none()
                }
            },
            _ => Task::none(),
        }
    }
}
