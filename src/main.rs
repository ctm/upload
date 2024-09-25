// This code silently fails on Firefox.  My guess is it's related to
// https://github.com/devashishdxt/rexie/issues/23 which suggests that
// serializing the files cause problems.  There's a chance that I can
// serialize the object_url instead (it's just a string) and have
// something that works with Firefox as well, without jumping through
// hoops to do additional serialization.
//
// FWIW, this code works fine with Brave, Edge and Safari. :-(

use {
    gloo_events::EventListener,
    gloo_utils::document,
    indexed_db::{Database, Error, Factory},
    log::{error, info},
    wasm_bindgen::JsCast,
    web_sys::{Blob, File, HtmlInputElement, Url},
    yew::{html::Scope, platform::spawn_local, prelude::*},
};

const DB_NAME: &str = "mb";
const INDEX: &str = "file";
const BUTTONS: &str = "buttons";

type OurError = ();

async fn build_database(link: Scope<App>) {
    let factory = match Factory::<OurError>::get() {
        Ok(f) => f,
        Err(e) => {
            error!("Can not get factory: {e:?}");
            return;
        }
    };

    match factory
        .open(DB_NAME, 1, |evt| async move {
            let db = evt.database();
            let store = db.build_object_store(BUTTONS).auto_increment().create()?;
            store
                .build_compound_index(INDEX, &["name", "lastModified", "size", "type"])
                .unique()
                .create()
                .inspect_err(|e| error!("could not build unique index: {e:?}"))?;
            Ok(())
        })
        .await
    {
        Err(_) => error!("Could not build buttons database"),
        Ok(db) => link.send_message(Msg::DbBuilt(db)),
    }
}

fn read_buttons(db: &Database<OurError>, link: Scope<App>) {
    let transaction = db.transaction(&STORE_NAMES).run(|t| async move {
        let store = t
            .object_store(BUTTONS)
            .inspect_err(|e| error!("Can't get store to read buttons: {e:?}"))?;
        let files = store
            .get_all(None)
            .await
            .inspect_err(|e| error!("reading buttons failed: {e:?}"))?;
        let buttons = files
            .into_iter()
            .filter_map(|file| match file.dyn_ref::<Blob>() {
                None => {
                    error!("Could not turn {file:?} into Blob");
                    None
                }
                Some(blob) => Url::create_object_url_with_blob(blob)
                    .inspect_err(|e| error!("Could not turn {blob:?} into object_url: {e:?}"))
                    .ok(),
            })
            .collect();
        link.send_message(Msg::ButtonsRead(buttons));
        Ok(())
    });
    spawn_local(async move {
        if let Err(e) = transaction.await {
            error!("Could not read buttons: {e:?}");
        }
    });
}

fn store_button(db: &Database<OurError>, file: File) {
    let transaction = db.transaction(&STORE_NAMES).rw().run(|t| async move {
        let store = t
            .object_store(BUTTONS)
            .inspect_err(|e| error!("Can't get store to read buttons: {e:?}"))?;
        store.add(&file).await.inspect_err(|e| { // TODO: don't use inspect_err
            if let Error::AlreadyExists = e {
                info!("That button is already stored");
            } else {
                error!("Could not store button: {e:?}");
            }
        })
    });
    spawn_local(async move {
        if let Err(e) = transaction.await {
            error!("Could not store buttons: {e:?}");
        }
    });
}

#[derive(Default)]
struct App {
    change_listener: Option<EventListener>,
    db: Option<Database<OurError>>,
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
    DbBuilt(Database<OurError>),
    ButtonsRead(Vec<String>),
    Clicked(ClickAction),
    StoreButton(File),
}

impl From<MouseEvent> for Msg {
    fn from(event: MouseEvent) -> Self {
        Msg::Clicked(event.into())
    }
}

static STORE_NAMES: [&str; 1] = [BUTTONS];

impl App {
    fn upload_image(&mut self, link: Scope<Self>) {
        let element = match document().create_element("input") {
            Ok(element) => element,
            Err(e) => {
                error!("Could not create input element: {e:?}");
                return;
            }
        };
        let input = match element.dyn_into::<HtmlInputElement>() {
            Ok(input) => input,
            Err(input) => {
                error!("Could not turn {input:?} into HtmlInputElement");
                return;
            }
        };
        if let Err(e) = input.set_attribute("type", "file") {
            error!("Could not set {input:?}'s type to file: {e:?}");
            return;
        }
        if let Err(e) = input.set_attribute("accept", "image/*") {
            error!("Could not set {input:?}'s accept to image/*: {e:?}");
            return;
        }
        // NOTE: don't bother setting change_listener back to None,
        // after the handler has been triggered, because there's not
        // much of a leak if we leave it in place.  After all, if we
        // create a new listener, it'll overwrite--and hence drop--the
        // old one, so at most we waste the space of one unneeded
        // listener.
        self.change_listener = Some(EventListener::once(
            &input,
            "change",
            move |e: &Event| match e.target() {
                None => error!("{e:?} has no target"),
                Some(target) => match target.dyn_into::<HtmlInputElement>() {
                    Err(target) => error!("Could not change {target:?} into HtmlInputElement"),
                    Ok(input) => match input.files() {
                        None => info!("No files"),
                        Some(files) => {
                            if let Some(file) = files.get(0) {
                                link.send_message(Msg::StoreButton(file));
                            } else {
                                info!("No file selected");
                            }
                        }
                    },
                },
            },
        ));
        input.click();
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

    fn add(&mut self, mut buttons: Vec<String>) -> bool {
        if buttons.is_empty() {
            false
        } else {
            self.button_face = ButtonFace::Custom(self.custom_faces.len());
            self.custom_faces.append(&mut buttons);
            true
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

    // Perhaps Button should be an actual Component itself, but since
    // this is just me futzing around with rexie, I didn't bother.
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
        spawn_local(build_database(ctx.link().clone()));
        Default::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.button.view(ctx.link())
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use {ClickAction::*, Msg::*};

        match msg {
            Clicked(Flip) => {
                self.button.incr();
                true
            }
            Clicked(ChooseImage) => {
                self.upload_image(ctx.link().clone());
                true
            }
            StoreButton(file) => {
                if let Ok(url) = Url::create_object_url_with_blob(&file) {
                    self.add_custom_button(url);
                }
                if let Some(db) = &self.db {
                    store_button(db, file);
                }
                true
            }
            DbBuilt(db) => {
                read_buttons(&db, ctx.link().clone());
                self.db = Some(db);
                false
            }
            ButtonsRead(buttons) => self.button.add(buttons),
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
