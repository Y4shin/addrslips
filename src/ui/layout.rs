use crate::core::db::{AreaDb, AreaRepository, ProjectDb};
use crate::Route;
use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

/// Project layout - loads the database and provides it to child routes via context
#[component]
pub fn ProjectLayout(file: String) -> Element {
    // Signal to hold the loaded database
    let mut db_signal: Signal<Option<Arc<ProjectDb>>> = use_signal(|| None);
    // Signal to hold any loading error
    let mut error_signal: Signal<Option<String>> = use_signal(|| None);

    // Provide the signal as context so child components can access it
    use_context_provider(|| db_signal);

    // Load the ProjectDb asynchronously based on the file path
    let _load_task = use_resource(move || {
        let file_path = file.clone();
        async move {
            // URL-decode the path (e.g., %2F -> /, %3A -> :)
            let decoded = urlencoding::decode(&file_path)
                .map(|s| s.into_owned())
                .unwrap_or(file_path);
            let path = PathBuf::from(decoded);
            match ProjectDb::new(&path).await {
                Ok(db) => db_signal.set(Some(Arc::new(db))),
                Err(e) => error_signal.set(Some(e.to_string())),
            }
        }
    });

    // Check current state
    let has_db = db_signal.read().is_some();
    let error = error_signal.read().clone();

    if has_db {
        // Database loaded - render provider wrapper with children
        rsx! {
            ProjectContextProvider {}
        }
    } else if let Some(error_msg) = error {
        rsx! {
            div {
                class: "error",
                h1 { "Error loading project" }
                p { "{error_msg}" }
                Link {
                    to: Route::Home {},
                    "Back to Home"
                }
            }
        }
    } else {
        rsx! {
            div {
                class: "loading",
                p { "Loading project..." }
            }
        }
    }
}

/// Inner component that provides the unwrapped database context
#[component]
fn ProjectContextProvider() -> Element {
    // Get the database signal from parent context
    let parent_db_signal: Signal<Option<Arc<ProjectDb>>> = use_context();
    let db_signal = use_signal(|| {
        parent_db_signal
            .read()
            .as_ref()
            .cloned()
            .expect("Database should be loaded in ProjectLayout")
    });

    // Provide the unwrapped Arc<ProjectDb> directly as context for child routes
    use_context_provider(|| db_signal);

    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
pub fn AreaLayout(file: String, area_id: i64) -> Element {
    // Get the database signal from context
    let db_signal = use_context::<Signal<Arc<ProjectDb>>>();
    // Signal to hold the loaded area database
    let mut area_db_signal: Signal<Option<Arc<AreaDb>>> = use_signal(|| None);
    // Signal to hold any loading error
    let mut error_signal: Signal<Option<String>> = use_signal(|| None);
    let _area_signal = use_resource(move || async move {
        let db_c = db_signal.read().clone();
        match db_c.get_area_repo(area_id).await {
            Ok(area_db) => area_db_signal.set(Some(Arc::new(area_db))),
            Err(e) => error_signal.set(Some(e.to_string())),
        }
    });
    // Check current state
    let has_area_db = area_db_signal.read().is_some();
    let error = error_signal.read().clone();
    if has_area_db {
        // Area database loaded - render provider wrapper with children
        rsx! {
            AreaContextProvider {}
        }
    } else if let Some(error_msg) = error {
        rsx! {
            div {
                class: "error",
                h1 { "Error loading area" }
                p { "{error_msg}" }
                Link {
                    to: Route::Home {},
                    "Back to Home"
                }
            }
        }
    } else {
        rsx! {
            div {
                class: "loading",
                p { "Loading area..." }
            }
        }
    }
}

/// Inner component that provides the unwrapped area database context
#[component]
fn AreaContextProvider() -> Element {
    // Get the area database signal from parent context
    let parent_area_db_signal: Signal<Option<Arc<AreaDb>>> = use_context();
    let area_db_signal = use_signal(|| {
        parent_area_db_signal
            .read()
            .as_ref()
            .cloned()
            .expect("Area Database should be loaded in AreaLayout")
    });

    // Provide the unwrapped Arc<AreaDb> directly as context for child routes
    use_context_provider(|| area_db_signal);

    rsx! {
        Outlet::<Route> {}
    }
}
