// This code currently stores buttons that people click on, but
// doesn't read from the store.

use {
    gloo_events::EventListener,
    gloo_utils::document,
    rexie::{Error as RexieError, Index, ObjectStore, Rexie, Transaction, TransactionMode},
    wasm_bindgen::JsCast,
    web_sys::{File, HtmlInputElement, Url},
    yew::{html::Scope, prelude::*},
};

const DB_NAME: &str = "mb";
const KEY: &str = "id";
const INDEX: &str = "file";
const BUTTONS: &str = "buttons";

async fn build_database() -> Msg {
    Msg::DbBuilt(
        Rexie::builder(DB_NAME)
            .version(1)
            .add_object_store(
                ObjectStore::new(BUTTONS)
                    .key_path(KEY)
                    .auto_increment(true)
                    .add_index(
                        Index::new_array(INDEX, ["name", "lastModified", "size", "type"])
                            .unique(true),
                    ),
            )
            .build()
            .await,
    )
}

async fn store_button(t: Transaction, file: File) -> Msg {
    async fn inner(t: Transaction, file: File) -> Result<(), RexieError> {
        let store = t.store(BUTTONS)?;
        store.add(&file, None).await.inspect_err(|e| {
            if let RexieError::IdbError(idb::Error::DomException(d)) = e {
                if d.name() == "ConstraintError" && d.message().contains("uniqueness") {
                    panic!("got it");
                }
            }
        })?;
        t.done().await?;
        Ok(())
    }
    Msg::ButtonStored(inner(t, file).await)
}

#[derive(Default)]
struct App {
    change_listener: Option<EventListener>,
    db: Option<Result<Rexie, RexieError>>,
    button: Button,
}

enum ClickAction {
    Flip,
    ChooseImage,
}

impl From<MouseEvent> for ClickAction {
    fn from(event: MouseEvent) -> Self {
        if event.shift_key() {
            Self::ChooseImage
        } else {
            Self::Flip
        }
    }
}

enum Msg {
    DbBuilt(Result<Rexie, RexieError>),
    Clicked(ClickAction),
    StoreButton(File),
    ButtonStored(Result<(), RexieError>),
}

impl From<MouseEvent> for Msg {
    fn from(event: MouseEvent) -> Self {
        Msg::Clicked(event.into())
    }
}

static STORE_NAMES: [&str; 1] = [BUTTONS];

impl App {
    fn upload_image(&mut self, link: Scope<Self>) -> Option<()> {
        let input = document()
            .create_element("input")
            .ok()?
            .dyn_into::<HtmlInputElement>()
            .ok()?;
        input.set_attribute("type", "file").ok()?;
        input.set_attribute("accept", "image/*").ok()?;
        // NOTE: we never attempt to set change_listener back to None,
        // because there's not much of a leak if we leave it in place,
        // since if we create a new listener, it'll overwrite--and
        // hence drop--the old one.
        self.change_listener = Some(EventListener::once(&input, "change", move |e: &Event| {
            if let Some(target) = e.target() {
                if let Ok(input) = target.dyn_into::<HtmlInputElement>() {
                    if let Some(files) = input.files() {
                        if let Some(file) = files.get(0) {
                            link.send_message(Msg::StoreButton(file));
                        }
                    }
                }
            }
        }));
        input.click();
        None
    }

    fn add_custom_button(&mut self, url: String) {
        self.button.add_custom(url);
    }
}

#[derive(Default)]
enum ButtonFace {
    #[default]
    Top,
    Bottom,
    Custom(usize),
}

impl ButtonFace {
    fn incr(&mut self, faces: &[String]) {
        use ButtonFace::*;

        *self = match self {
            Top => Bottom,
            Bottom if faces.is_empty() => Top,
            Bottom => Custom(0),
            Custom(i) if *i < faces.len() - 1 => Custom(*i + 1),
            Custom(_) => Top,
        }
    }
}

#[derive(Default)]
struct Button {
    button_face: ButtonFace,
    custom_faces: Vec<String>,
}

impl Button {
    fn incr(&mut self) {
        self.button_face.incr(&self.custom_faces)
    }

    fn add_custom(&mut self, url: String) {
        match self.custom_faces.iter().position(|face| face == &url) {
            Some(i) => self.button_face = ButtonFace::Custom(i),
            None => {
                self.button_face = ButtonFace::Custom(self.custom_faces.len());
                self.custom_faces.push(url);
            }
        }
    }

    fn class_and_style(&self) -> (&'static str, Option<String>) {
        use ButtonFace::*;

        match &self.button_face {
            Top => ("button-wrapper examine", None),
            Bottom => ("button-wrapper examine flipped", None),
            Custom(i) => (
                "button-wrapper examine",
                Some(format!(
                    "background-image: url(\"{}\")",
                    self.custom_faces[*i]
                )),
            ),
        }
    }

    // Button should probably be an actual Component
    fn view(&self, link: &Scope<App>) -> Html {
        let onclick: Callback<MouseEvent> = link.callback(Into::<Msg>::into);
        let (class, style) = self.class_and_style();
        html! {
            <div class={"button"}>
                <div {class} {style} {onclick} />
            </div>
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(build_database());
        Default::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.button.view(ctx.link())
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use ClickAction::*;

        match msg {
            Msg::Clicked(Flip) => {
                self.button.incr();
                true
            }
            Msg::Clicked(ChooseImage) => {
                self.upload_image(ctx.link().clone());
                true
            }
            Msg::StoreButton(file) => {
                if let Ok(url) = Url::create_object_url_with_blob(&file) {
                    self.add_custom_button(url);
                }
                if let Some(Ok(db)) = &self.db {
                    if let Ok(t) = db.transaction(&STORE_NAMES, TransactionMode::ReadWrite) {
                        ctx.link().send_future(store_button(t, file));
                    }
                }
                true
            }
            Msg::DbBuilt(result) => {
                self.db = Some(result);
                false
            }
            Msg::ButtonStored(_) => false, // TODO
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
