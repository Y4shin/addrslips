use dioxus::prelude::*;
 
 /// Address Detection page

#[component]
pub fn AddressDetection(file: String, area_id: i64) -> Element {
    rsx! {
        div {
            id: "address-detection",
            h1 { "Address Detection" }
            p { "Area ID: {area_id}" }
        }
    }
}