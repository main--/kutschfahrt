#![recursion_limit = "1024"]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use console_error_panic_hook::set_once as set_panic_hook;
use gloo_utils::window;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;
use yew_router::prelude::*;

use web_protocol::MyState;

mod ingame;


#[derive(Clone, Debug, PartialEq, Routable)]
pub enum AppRoute {
    #[at("/game/:id")]
    Game { id: String },
    #[at("/")]
    Home,
}
type Link = yew_router::prelude::Link<AppRoute>;




struct App {
    my_state: Option<MyState>,
}


enum Msg {
    GotState(MyState),
    Login,
    Logout,
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
    let _resp = JsFuture::from(window().fetch_with_request(&request)).await.unwrap();
    /*let resp: web_sys::Response = resp.dyn_into().unwrap();
    let text = JsFuture::from(resp.text().unwrap()).await.unwrap();
    let text = text.as_string().unwrap();
    serde_json::from_str(&text).unwrap()*/
}

fn view_game_item(game: String) -> Html {
    html! { <li><Link to={AppRoute::Game { id: game.clone() }}>{game}</Link></li> }
}
fn view_content(r: &AppRoute, my_games: Vec<String>) -> Html {
    match r {
        AppRoute::Home => html! {
            <div>
                <h3 class="title">{"Your Games"}</h3>
                <div class="content">
                    <ul>
                        {for my_games.into_iter().map(|g| view_game_item(g))}
                        <li><Link to={AppRoute::Game { id: uuid::Uuid::new_v4().to_string() }}>{"+ New Game"}</Link></li>
                    </ul>
                </div>
            </div>
        },
        AppRoute::Game { id: g } => html! {
            <div class="content">
                <h1>{format!("Game '{}'", g)}</h1>
                <div>
                    <ingame::Ingame game={g.clone()} />
                </div>
            </div>
        },
    }
}
impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async { Msg::GotState(fetch_json("/api/me").await) });
        App { my_state: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GotState(s) => {
                self.my_state = Some(s);
            }
            Msg::Login => {
                let loc = window().location();
                let url = format!("/api/login?returnurl={}", loc.origin().unwrap());
                loc.set_href(&url).unwrap();
            }
            Msg::Logout => {
                window().location().set_pathname("/api/logout").unwrap();
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let login_btn = match self.my_state {
            None => html! { <div /> },
            Some(MyState::LoggedOut) => html! { <button class="button is-black is-outlined" onclick={ctx.link().callback(|_| Msg::Login)}>{"Login"}</button> },
            Some(MyState::LoggedIn { .. }) => html! { <button class="button is-black is-outlined" onclick={ctx.link().callback(|_| Msg::Logout)}>{"Logout"}</button> },
        };
        let (my_games, logged_in) = match &self.my_state {
            Some(MyState::LoggedIn { my_games }) => (my_games.clone(), true),
            _ => (vec![], false),
        };
        html! {
            <BrowserRouter>
                <nav class="navbar is-success">
                    <div class="container">
                        <div class="navbar-brand">
                            <div class="navbar-item">
                                <h3 class="title has-text-white is-4">{"Kutschfahrt"}</h3>
                            </div>
                        </div>
                        <div class="navbar-menu">
                            <div class="navbar-end">
                                <div class="navbar-item">
                                    {login_btn}
                                </div>
                            </div>
                        </div>
                    </div>
                </nav>

                <div class="container is-centered">
                    {if logged_in { html! {
                        <Switch<AppRoute>
                            render={Switch::render(move |r| view_content(r, my_games.clone()))}
                        />
                    } } else { html! {
                        {"Please log in."}
                    } }}
                </div>
            </BrowserRouter>
        }
    }
}

fn main() {
    set_panic_hook();
    yew::start_app::<App>();
}

