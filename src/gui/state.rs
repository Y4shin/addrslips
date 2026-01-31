// use crate::core::Project;  // Will be available in Phase 2

#[derive(Debug, Clone)]
pub struct AppState {
    // pub current_project: Option<Project>,  // Will be enabled in Phase 2
    pub recent_projects: Vec<String>,  // File paths
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            // current_project: None,
            recent_projects: Vec::new(),
        }
    }
}
