// Make it so that shift-clicking brings up an image picker
// Ase an ObjectURL to update the image
//
// See https://developer.mozilla.org/en-US/docs/Web/API/File_API/Using_files_from_web_applications
// for more info

use wasm_bindgen::JsCast;
use web_sys::Element;
use yew::prelude::*;

const FLIPPED: &str = "flipped";

fn toggle_flipped(e: &MouseEvent) -> Option<()> {
    let cl = e.target()?.dyn_into::<Element>().ok()?.class_list();
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
            toggle_flipped(&e);
        }
    }
    .into();
    html! {
        <div class={"button"}>
            <div class={"button-wrapper examine"} {onclick}/>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
