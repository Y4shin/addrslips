use std::{path::PathBuf, sync::Arc};

use dioxus::prelude::*;

use crate::{
    Route, core::db::{AreaRepository, Color, NewArea, ProjectDb}, ui::components::{ColorPicker, Loading}
};

/// Project overview page
#[component]
pub fn ProjectOverview(file: String) -> Element {
    // Get the database from context (Signal<Option<Arc<ProjectDb>>>)
    let db_signal: Signal<Arc<ProjectDb>> = use_context();
    //let has_db = db.is_some();
    let mut areas_signal = use_resource(move || async move {
        let db_c = db_signal.read().clone();
        db_c.get_areas().await.map_err(|e| e.to_string())
    });
    rsx! {
        div {
            id: "project-overview",
            h1 { "Project Overview" }
            p { "File: {file}" }
            match areas_signal.cloned() {
                None => rsx! { Loading {}},
                Some(Err(error_msg)) => {
                    rsx! {
                        div {
                            class: "error",
                            h2 { "Error loading areas" }
                            p { "{error_msg}" }
                        }
                    }
                },
                Some(Ok(areas)) => {
                    let mut new_area_name = use_signal(|| "".to_string());
                    let mut new_area_color = use_signal(|| Color::WHITE);
                    let mut new_area_image = use_signal(|| None as Option<PathBuf>);
                    let mut add_error = use_signal(|| None as Option<String>);
                    rsx! {
                        div {
                            h2 { "Areas" }
                            ul {
                                for area in areas {
                                    li {
                                        Link {
                                            to: Route::AddressDetection { file: file.clone(), area_id: area.id },
                                            "{area.name}"
                                        }
                                    }
                                }
                            }
                            form {
                                onsubmit: move |evt| {
                                    evt.prevent_default();
                                    let db = db_signal();
                                    let new_area_name_val = new_area_name();
                                    let new_area_color_val = new_area_color();
                                    let new_area_image_val = new_area_image();
                                    async move {
                                        if let Err(e) = add_area(db, new_area_name_val, new_area_color_val, new_area_image_val).await {
                                            add_error.set(Some(e.to_string()));
                                        } else {
                                            // Clear form on success
                                            new_area_name.set("".to_string());
                                            new_area_color.set(Color::WHITE);
                                            new_area_image.set(None);
                                            add_error.set(None);
                                            areas_signal.restart();
                                        }
                                    }
                                },
                                input {
                                    r#type: "text",
                                    placeholder: "Area Name",
                                    value: "{new_area_name()}",
                                    oninput: move |e| new_area_name.set(e.value())
                                }
                                ColorPicker {
                                    selected_color: new_area_color
                                }
                                input {
                                    type: "file",
                                    accept: ".png,.jpg,.jpeg",
                                    multiple: false,
                                    onchange: move |e| {
                                        let file = e.files().get(0).cloned();
                                        if let Some(file) = file {
                                            new_area_image.set(Some(file.path()));
                                        } else {
                                            new_area_image.set(None);
                                        }
                                    }
                                }
                                input { r#type: "submit" }
                            }
                            if let Some(error) = add_error() {
                                p {
                                    class: "error",
                                    "{error}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn add_area(db: Arc<ProjectDb>, name: String, color: Color, image: Option<PathBuf>) -> anyhow::Result<()> {
    let image_path = image.ok_or_else(|| anyhow::anyhow!("Image path is required"))?;
    let new_area = NewArea {
        name,
        color,
        image_path,
    };
    db.add_area(new_area).await?;
    Ok(())
}
