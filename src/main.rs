use dioxus::prelude::*;
pub mod core;
pub mod detection;
pub mod models;
pub mod pipeline;
pub mod ui;

use crate::ui::{
    address_detection::AddressDetection,
    home::Home,
    layout::{AreaLayout, ProjectLayout},
    overview::ProjectOverview,
};

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Home {},

    #[nest("/project/:file")]
        #[layout(ProjectLayout)]
        #[route("/")]
        ProjectOverview { file: String },
        #[nest("/area/:area_id")]
            #[layout(AreaLayout)]
            #[route("/address-detection")]
            AddressDetection { file: String, area_id: i64 },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}
