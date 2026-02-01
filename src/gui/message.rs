use crate::gui::{
    screens::{
        ScreenData, ScreenMessage, landing_page::LandingPageScreen,
        loading_page::LoadingPageScreen, select_area::SelectAreaScreen,
    },
    state::ProjectState,
};

#[derive(Debug)]
pub enum Message {
    LandingPage(ScreenMessage<LandingPageScreen>),
    LoadingPageMessage(ScreenMessage<LoadingPageScreen>),
    SelectAreaMessage(ScreenMessage<SelectAreaScreen>),
    ChangeScreen(ScreenData),
    LoadProject(ProjectState<'static>, SelectAreaScreen),
}
