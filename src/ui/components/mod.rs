use dioxus::{html::g::color, prelude::*};

use crate::core::db::Color;

/// A loading spinner component
#[component]
pub fn Loading() -> Element {
    rsx! {
        div {
            class: "loading-spinner",
            "Loading..."
        }
    }
}

#[derive(Clone, Props, PartialEq, Eq)]
pub struct ColorPickerProps {
    pub selected_color: Signal<Color>,
}

#[component]
pub fn ColorPicker(mut props: ColorPickerProps) -> Element {
    let mut color_str = use_signal(|| props.selected_color.read().to_hex_string());
    let computed_color =
        use_memo(move || Color::from_hex_string(color_str().as_str()).map_err(|e| e.to_string()));
    use_effect(move || match computed_color() {
        Ok(col) => {
            props.selected_color.set(col);
        }
        Err(_) => {}
    });
    rsx! {
        div {
            class: "color-picker",
            input {
                value: "{color_str().to_ascii_uppercase()}",
                oninput: move |event| color_str.set(event.value())
            }
            span {
                style: "display: inline-block; width: 24px; height: 24px; border: 1px solid #000; background-color: {props.selected_color.read().to_hex_string()}; margin-left: 8px;",
            }
            if let Err(e) = computed_color() {
                div {
                    class: "error",
                    "{e}"
                }
            }
        }
    }
}
