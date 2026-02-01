// use crate::core::Project;  // Will be available in Phase 2

use crate::core::db::{AreaDb, ProjectDb};


#[derive(Debug)]
pub struct ProjectState<'a> {
    pub project_db: ProjectDb,
    pub area_db: Option<AreaDb<'a>>,
}

#[derive(Debug)]
pub struct AppState {
    pub current_project: Option<ProjectState<'static>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_project: None,
        }
    }
}
