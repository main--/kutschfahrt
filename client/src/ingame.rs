use gloo_timers::future::TimeoutFuture;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use web_protocol::GameInfo;

pub struct Ingame {
    game: String,
    game_info: Option<GameInfo>,
    command: String,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub game: String,
}

pub enum Msg {
    Refresh(GameInfo),
    TypeCommand(String),
    Submit,
}

async fn update_state(s: String) -> Msg {
    //JsFuture::from(sleep(1000)).await.unwrap();
    TimeoutFuture::new(1000).await;
    let path = format!("/api/game/{}", s);
    Msg::Refresh(super::fetch_json(&path).await)
}

impl Ingame {
    fn trigger_refresh(&self, ctx: &Context<Self>) {
        let game = self.game.clone();
        ctx.link().send_future(async move { update_state(game).await });
    }
}
impl Component for Ingame {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let i = Ingame {
            game: ctx.props().game.clone(),
            game_info: None,
            command: String::new(),
        };
        i.trigger_refresh(ctx);
        i
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Refresh(info) => {
                self.game_info = Some(info);
                self.trigger_refresh(ctx);
            }
            Msg::TypeCommand(s) => {
                gloo_console::log!("typecommand", &s);
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

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <pre>
                    {serde_json::to_string_pretty(&self.game_info).unwrap()}
                </pre>
                <input
                    value={self.command.clone()}
                    oninput={ctx.link().callback(|e: InputEvent| { let input: HtmlInputElement = e.target_unchecked_into(); Msg::TypeCommand(input.value()) })}
                    onkeypress={ctx.link().batch_callback(|e: KeyboardEvent| {
                        if e.key() == "Enter" {
                            vec![Msg::Submit]
                        } else {
                            vec![]
                        }
                    })}
                />
            </>
        }
    }
}
