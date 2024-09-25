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
    log::{error, info},
    indexed_db::{Error, Factory, Index, ObjectStore, Database, Transaction},
    wasm_bindgen::JsCast,
    web_sys::{Blob, File, HtmlInputElement, Url},
    yew::{html::Scope, platform::spawn_local, prelude::*},
};

const DB_NAME: &str = "mb";
const KEY: &str = "id";
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
   
    match factory.open(DB_NAME, 1, |evt| async move {
        let db = evt.database();
        let store = db.build_object_store(BUTTONS)
            .auto_increment()
            .create()?;
        store.build_compound_index(INDEX, &["name", "lastModified", "size", "type"]).unique().create().inspect_err(|e| error!("could not build unique index"))?;
        Ok(())
    }).await {
        Err(_) => error!("Could not build buttons database"),
        Ok(db) => link.send_message(Msg::DbBuilt(db)),
    }
}

async fn async_read_buttons(store: ObjectStore<OurError>, link: Scope<App>) {
    match store.get_all(None).await {
        Err(e) => error!("reading buttons failed: {e:?}"),
        Ok(files) => {
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
        }
    }
}

fn read_buttons(db: &Database<OurError>, link: Scope<App>) {
    /*
    let transaction = match db.transaction(&STORE_NAMES, TransactionMode::ReadOnly) {
        Ok(t) => t,
        Err(e) => {
            error!("Can't create transaction to read buttons: {e:?}");
            return;
        }
    };
    let store = match transaction.store(BUTTONS) {
        Ok(s) => s,
        Err(e) => {
            error!("Can't get store to read buttons: {e:?}");
            return;
        }
    };
    spawn_local(async_read_buttons(store, link));
    */
}

// If we wanted to, we could split this into a non-async store_button
// and an async async_store_button, like we do with read_buttons and
// async_read_buttons above.  The upside to doing the split is that
// nothing has to be added to the executor in the case where there's
// an error before anything async is called. That's not much of an
// upside though if the error is unlikely to occur and time isn't
// critical.
//
// So, the reason read_buttons is split and store_button isn't is just
// due to me fooling around, since I'm not particularly proficient in
// async rust.
async fn store_button(t: Transaction<OurError>, file: File) {
    /*
    let store = match t.store(BUTTONS) {
        Ok(s) => s,
        Err(e) => {
            error!("Can't get store to store buttons: {e:?}");
            return;
        }
    };
    match store.add(&file, None).await {
        Ok(_) => {
            // Do not call done if store failed, because we'll get a panic.
            if let Err(e) = t.done().await {
                error!("Could not complete button storage transaction: {e:?}");
            }
        }
        Err(e) => {
            if let Error::IdbError(idb::Error::DomException(d)) = e {
                // I am not particularly happy about this code to detect a
                // uniqueness constraint violation, but it appears to work
                if d.name() == "ConstraintError" && d.message().contains("uniqueness") {
                    info!("That button is already stored");
                }
            } else {
                error!("Could not store button: {e:?}");
            }
        }
    }
    */
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
                    /*
                    if let Ok(t) = db.transaction(&STORE_NAMES, TransactionMode::ReadWrite) {
                        spawn_local(store_button(t, file));
                    }
                    */
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
