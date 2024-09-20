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
    wasm_bindgen::{JsCast, JsValue},
    web_sys::{HtmlElement, HtmlInputElement, Url},
    yew::prelude::*,
};

const FLIPPED: &str = "flipped";

fn upload_image(button: &HtmlElement) -> Option<()> {
    let button_style = button.style();
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

fn toggle_flipped(button: &HtmlElement) -> Option<()> {
    let _ = button.style().remove_property("background-image");
    let cl = button.class_list();
    if cl.contains(FLIPPED) {
        let _ = cl.remove_1(FLIPPED);
    } else {
        let _ = cl.add_1(FLIPPED);
    }
    None
}

#[derive(Default)]
struct App {
    change_listener: Option<EventListener>,
}

enum ClickError {
    NoTarget,
    NotHtmlElement,
}

enum ClickAction {
    Flip,
    ChooseImage,
}

struct Click {
    button: HtmlElement,
    action: ClickAction,
}

type ClickAttempt = Result<Click, ClickError>;

impl From<&MouseEvent> for ClickAction {
    fn from(event: &MouseEvent) -> Self {
        if event.shift_key() {
            Self::ChooseImage
        } else {
            Self::Flip
        }
    }
}

enum Msg {
    Clicked(ClickAttempt),
}

impl TryFrom<&MouseEvent> for Click {
    type Error = ClickError;

    fn try_from(m: &MouseEvent) -> ClickAttempt {
        let button = m
            .target()
            .ok_or(ClickError::NoTarget)?
            .dyn_into::<HtmlElement>()
            .map_err(|_| ClickError::NotHtmlElement)?;
        let action = m.into();
        Ok(Click { button, action })
    }
}

fn clicked(m: MouseEvent) -> Msg {
    Msg::Clicked((&m).try_into())
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick: Callback<MouseEvent> = ctx.link().callback(clicked);
        html! {
            <div class={"button"}>
                <div class={"button-wrapper examine"} {onclick} />
                </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use ClickAction::*;

        match msg {
            Msg::Clicked(Err(e)) => false, // TODO
            Msg::Clicked(Ok(Click {
                action: Flip,
                button,
            })) => {
                toggle_flipped(&button);
                true
            }
            Msg::Clicked(Ok(Click {
                action: ChooseImage,
                button,
            })) => {
                upload_image(&button);
                true
            }
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
