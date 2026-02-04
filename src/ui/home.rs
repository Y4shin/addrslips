use dioxus::prelude::*;
use rfd::AsyncFileDialog;
use crate::Route;

/// Home page
#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            id: "home",
            h1 { "Address Slips" }
            p { "Select or create a project to get started." }
            button {
                onclick: move |_| {
                    spawn(open_path(true));
                },
                "Create New Project"
            }
            button {
                onclick: move |_| {
                    spawn(open_path(false));
                },
                "Open Existing Project"
            }
        }
    }
}

async fn open_path(create: bool) -> () {
    let file_picker = AsyncFileDialog::new()
        .add_filter("Address Slips Project", &["asl"])
        .set_title(if create { "Create New Project" } else { "Open Project" });
    let file = if create {
        file_picker.save_file().await
    } else {
        file_picker.pick_file().await
    };
    if let Some(file) = file {
        let path = file.path();
        let encoded_path = urlencoding::encode(path.to_string_lossy().as_ref()).into_owned();
        navigator().push(Route::ProjectOverview { file: encoded_path });
    }
}