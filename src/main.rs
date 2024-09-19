// See https://github.com/rustwasm/wasm-bindgen/issues/2551 for useful stuff
// https://github.com/devashishdxt/rexie
//
// Looks like rexie stores JsValues and that a Blob is a JsValue, but I
// can't be sure.

// Figure out how to save it via local storage
//
// See https://developer.mozilla.org/en-US/docs/Web/API/File_API/Using_files_from_web_applications
// for more info

use {
    gloo_events::EventListener,
    gloo_utils::document,
    wasm_bindgen::JsCast,
    web_sys::{HtmlElement, HtmlInputElement, Url},
    yew::prelude::*,
};

const FLIPPED: &str = "flipped";

fn upload_image(e: &MouseEvent) -> Option<()> {
    let button_style = e.target()?.dyn_into::<HtmlElement>().ok()?.style();
    let input = document()
        .create_element("input")
        .ok()?
        .dyn_into::<HtmlInputElement>()
        .ok()?;
    input.set_attribute("type", "file").ok()?;
    input.set_attribute("accept", "image/*").ok()?;
    EventListener::once(&input, "change", move |e: &Event| {
        if let Some(target) = e.target() {
            if let Ok(input) = target.dyn_into::<HtmlInputElement>() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        if let Ok(url) = Url::create_object_url_with_blob(&file) {
                            let _ = button_style
                                .set_property("background-image", &format!("url(\"{url}\")"));
                        }
                        // TODO: spawn a future that gets an array_buffer
                        // and when we get that array buffer, store it in
                        // local storage for now
                    }
                }
            }
        }
    })
    .forget(); // NOTE: forget is for PoC, not production
    input.click();
    None
}

fn toggle_flipped(e: &MouseEvent) -> Option<()> {
    let button = e.target()?.dyn_into::<HtmlElement>().ok()?;
    let _ = button.style().remove_property("background-image");
    let cl = button.class_list();
    if cl.contains(FLIPPED) {
        let _ = cl.remove_1(FLIPPED);
    } else {
        let _ = cl.add_1(FLIPPED);
    }
    None
}

#[function_component(App)]
fn app() -> Html {
    let onclick: Callback<MouseEvent> = {
        |e: MouseEvent| {
            if e.shift_key() {
                upload_image(&e);
            } else {
                toggle_flipped(&e);
            }
        }
    }
    .into();
    html! {
        <div class={"button"}>
            <div class={"button-wrapper examine"} {onclick} />
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
