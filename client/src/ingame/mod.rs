use gloo_console::log;
use gloo_timers::future::TimeoutFuture;
use web_sys::{HtmlInputElement};
use yew::prelude::*;
use web_protocol::{GameInfo, GameCommand, PerspectiveTurnState};

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
                <ContextProvider<Commander> context={Commander { game: self.game.clone() }}>
                    {self.game_info.clone().map(|g| html! { <GameUi gamestate={g} /> }).into_iter().collect::<Html>()}
                </ContextProvider<Commander>>
            </>
        }
    }
}


#[derive(Clone, PartialEq)]
struct Commander {
    game: String,
}
impl Commander {
    fn cmd(&self, cmd: GameCommand) {
        log!("Sending command", format!("{:?}", cmd));
        let path = format!("/api/game/{}", self.game);
        wasm_bindgen_futures::spawn_local(async move {
            super::post_json(&path, &cmd).await;
        });
    }
}


mod pregame;
mod turnstart;
mod trading;


#[derive(Properties, PartialEq)]
struct GameUiProps {
    pub gamestate: GameInfo,
}
#[function_component(GameUi)]
fn game_ui(props: &GameUiProps) -> Html {
    match &props.gamestate {
        GameInfo::WaitingForPlayers { players, you } => html! { <pregame::WaitingForPlayers players={players.clone()} you={you.clone()} /> },
        GameInfo::Game(p) => {
            let me = &p.players[p.your_player_index];
            let body = match &p.turn {
                PerspectiveTurnState::TurnStart { player } if player == &me.player => html! { <turnstart::MyTurnStart perspective={p.clone()} /> },
                PerspectiveTurnState::TurnStart { player } => html! { {format!("Waiting for {} ...", player)} },
                PerspectiveTurnState::GameOver { winner } => html! { <div class="victory-text">{format!("The {:?} is victorious!", winner)}</div> },
                &PerspectiveTurnState::TradePending { offerer, target, item } if target == me.player => html! { <trading::TradeOffer you={p.you.clone()} {offerer} item={item.unwrap()} stack_empty={p.item_stack == 0} /> },
                PerspectiveTurnState::TradePending { offerer, target, .. } => html! { <div class="trade-text">{format!("{} is offering an item to {} ...", offerer, target)}</div> },
                PerspectiveTurnState::ResolvingTradeTrigger { offerer, target, trigger } => todo!(),
                PerspectiveTurnState::Attacking { attacker, defender, state } => html! { {format!("{} is attacking {}", attacker, defender)} },
            };
            html! { <div class="hud">{body}</div> }
        }
    }
}
