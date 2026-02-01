use iced::{
    Element, Task,
    widget::{container, text},
};

use crate::{
    core::db::{Area, AreaRepository},
    gui::{
        AppState,
        screens::{Screen, ScreenMessage},
        state::ProjectState,
        widgets::{Step, layout},
    },
};

#[derive(Debug, Clone)]
pub struct SelectAreaScreen {
    areas: Vec<Area>,
}

#[derive(Debug, Clone)]
pub enum SelectAreaMessage {
    None,
}

#[derive(Debug, Clone)]
pub enum SelectAreaParentMessage {
    None,
}

impl Screen for SelectAreaScreen {
    type Message = SelectAreaMessage;
    type ParentMessage = SelectAreaParentMessage;

    fn view(&self) -> Element<'_, ScreenMessage<Self>> {
        layout(
            text("Sidebar"),
            text("Select Area Screen - Placeholder"),
            Step::CreateArea,
        )
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

impl SelectAreaScreen {
    pub async fn new(state: &ProjectState<'_>) -> Self {
        let areas = state
            .project_db
            .get_areas()
            .await
            .unwrap_or_else(|_| Vec::new());
        Self { areas }
    }
}
