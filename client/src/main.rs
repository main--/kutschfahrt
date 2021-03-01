#![recursion_limit = "1024"]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use console_error_panic_hook::set_once as set_panic_hook;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use ybc::TileCtx::{Ancestor, Child, Parent};
use yew::prelude::*;
use yew::utils::window;
//use yewtil::PureComponent;
use yewtil::future::LinkFuture;

use web_protocol::MyState;

mod ingame;


struct App {
    my_state: Option<MyState>,
    current_game: Option<String>,
    link: ComponentLink<Self>,
}


enum Msg {
    GotState(MyState),
    Login,
    Logout,

    GoToGame(Option<String>),
}

async fn fetch_json<T: serde::de::DeserializeOwned>(path: &str) -> T {
    let resp = JsFuture::from(window().fetch_with_str(path)).await.unwrap();
    let resp: web_sys::Response = resp.dyn_into().unwrap();
    let text = JsFuture::from(resp.text().unwrap()).await.unwrap();
    let text = text.as_string().unwrap();
    serde_json::from_str(&text).unwrap()
}
async fn post_json<T: serde::Serialize>(path: &str, body: &T) {
    let body = serde_json::to_string(body).unwrap();
    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    opts.body(Some(&JsValue::from(&body)));
    let request = web_sys::Request::new_with_str_and_init(path, &opts).unwrap();
    let resp = JsFuture::from(window().fetch_with_request(&request)).await.unwrap();
    /*let resp: web_sys::Response = resp.dyn_into().unwrap();
    let text = JsFuture::from(resp.text().unwrap()).await.unwrap();
    let text = text.as_string().unwrap();
    serde_json::from_str(&text).unwrap()*/
}

impl App {
    fn view_game_item(&self, game: String) -> Html {
        let game2 = game.clone();
        let link = self.link.callback_once(move |e: yew::events::MouseEvent| { e.prevent_default(); Msg::GoToGame(Some(game2)) });
        html! { <li><a onclick=link>{game}</a></li> }
    }
}
impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_future(async { Msg::GotState(fetch_json("/me").await) });
        App {
            my_state: None,
            current_game: None,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::GotState(s) => {
                self.my_state = Some(s);
            }
            Msg::Login => {
                window().location().set_pathname("/login").unwrap();
            }
            Msg::Logout => {
                window().location().set_pathname("/logout").unwrap();
            }
            Msg::GoToGame(g) => {
                self.current_game = g;
            }
        }
        true
    }

    fn change(&mut self, _: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let login_btn = match self.my_state {
            None => html! { <div /> },
            Some(MyState::LoggedOut) => html! { <ybc::Button classes="is-black is-outlined" onclick=self.link.callback(|_| Msg::Login)>{"Login"}</ybc::Button> },
            Some(MyState::LoggedIn { .. }) => html! { <ybc::Button classes="is-black is-outlined" onclick=self.link.callback(|_| Msg::Logout)>{"Logout"}</ybc::Button> },
        };
        let (my_games, logged_in) = match &self.my_state {
            Some(MyState::LoggedIn { my_games }) => (my_games.clone(), true),
            _ => (vec![], false),
        };
        let new_game = self.link.callback(move |e: yew::events::MouseEvent| {
            e.prevent_default();
            Msg::GoToGame(Some(uuid::Uuid::new_v4().to_string()))
        });
        let content = match &self.current_game {
            None => html! {
                <ybc::Section>
                    <ybc::Title>{"Your Games"}</ybc::Title>
                    <ybc::Content>
                        <ul>
                            {for my_games.into_iter().map(|g| self.view_game_item(g))}
                            {if logged_in { html! { <li><a onclick=new_game>{"+ New Game"}</a></li> } } else { html! { {"Please log in."} } }}
                        </ul>
                    </ybc::Content>
                </ybc::Section>
            },
            Some(s) => html! {
                <ybc::Section>
                    <ybc::Title>{format!("Game '{}'", s)}</ybc::Title>
                    <ybc::Content>
                        <ingame::Ingame game=s />
                    </ybc::Content>
                </ybc::Section>
            },
        };
        html! {
            <>
                <ybc::Navbar
                    classes="is-success"
                    padded=true
                    navbrand=html!{
                        <ybc::NavbarItem>
                            <ybc::Title classes="has-text-white" size=ybc::HeaderSize::Is4>{"Kutschfahrt"}</ybc::Title>
                        </ybc::NavbarItem>
                    }
                    navstart=html!{}
                    navend=html!{
                        <>
                            <ybc::NavbarItem>
                                {login_btn}
                            </ybc::NavbarItem>
                        </>
                    }
                />

                <ybc::Container classes="is-centered">
                    {content}
                </ybc::Container>
            </>
        }
    }
}

fn main() {
    set_panic_hook();
    yew::start_app::<App>();
}

