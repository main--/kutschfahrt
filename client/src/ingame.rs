use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use ybc::TileCtx::{Ancestor, Child, Parent};
use yew::prelude::*;
use yew::utils::window;
use yew::events::{InputData, KeyboardEvent};
use yewtil::future::LinkFuture;
use js_sys::Promise;
use web_protocol::GameInfo;

pub struct Ingame {
    game: String,
    game_info: Option<GameInfo>,
    command: String,
    link: ComponentLink<Self>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub game: String,
}

pub enum Msg {
    Refresh(GameInfo),
    TypeCommand(String),
    Submit,
}

async fn update_state(s: String) -> Msg {
    JsFuture::from(sleep(1000)).await.unwrap();
    let path = format!("/api/game/{}", s);
    Msg::Refresh(super::fetch_json(&path).await)
}

#[wasm_bindgen(inline_js = "export function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }")]
extern "C" {
    fn sleep(ms: i32) -> Promise;
}

impl Ingame {
    fn trigger_refresh(&self) {
        let game = self.game.clone();
        self.link.send_future(async move { update_state(game).await });
    }
}
impl Component for Ingame {
    type Message = Msg;
    type Properties = Props;

    fn create(p: Self::Properties, link: ComponentLink<Self>) -> Self {
        let i = Ingame {
            game: p.game,
            game_info: None,
            command: String::new(),
            link,
        };
        i.trigger_refresh();
        i
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Refresh(info) => {
                self.game_info = Some(info);
                self.trigger_refresh();
            }
            Msg::TypeCommand(s) => {
                self.command = s;
            }
            Msg::Submit => {
                let path = format!("/api/game/{}", &self.game);
                if let Ok(command) = serde_json::from_str::<serde_json::Value>(&self.command) {
                    self.command = String::new();
                    wasm_bindgen_futures::spawn_local(async move {
                        super::post_json(&path, &command).await;
                    });
                }
            }
        }
        true
    }

    fn change(&mut self, _: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <pre>
                {serde_json::to_string_pretty(&self.game_info).unwrap()}
                </pre>
                <input
                    value=&self.command
                    oninput=self.link.callback(|e: InputData| Msg::TypeCommand(e.value))
                    onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                        if e.key() == "Enter" {
                            vec![Msg::Submit]
                        } else {
                            vec![]
                        }
                    })
                />
            </>
        }
    }
}
