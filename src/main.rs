// Here's my plan:
// Add Adrienne's assets, with the regular button showing.
//   look up how to do trunk assets
//   look at how the button is displayed in mb2 so we get the right size
//   and zoom
// Make it so that clicking on it brings up the narcoleptic dinosaur
// Make it so that shift-clicking brings up an image picker
// Ase an ObjectURL to update the image
//
// See https://developer.mozilla.org/en-US/docs/Web/API/File_API/Using_files_from_web_applications
// for more info


use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class={"button"}>
            <div class={"button-wrapper examine"}/>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
