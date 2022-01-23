use std::rc::Rc;

use gloo_console::log;
use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, EventSource, MessageEvent};
use yew::prelude::*;
use web_protocol::{GameInfo, GameCommand, PerspectiveTurnState, Perspective};

pub struct Ingame {
    game: String,
    game_info: Option<GameInfo>,
    command: String,

    eventsrc: EventSource,
    _msg_listener: EventListener,
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

impl Component for Ingame {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let game = ctx.props().game.clone();
        let eventsrc = EventSource::new(&format!("/api/game/{}/events", game)).unwrap();

        let update_cb = ctx.link().callback(|gi| Msg::Refresh(gi));
        let _msg_listener = EventListener::new(&eventsrc, "message", move |event| {
            let event = event.dyn_ref::<MessageEvent>().unwrap();
            let text = event.data().as_string().unwrap();
            let gi = serde_json::from_str::<GameInfo>(&text).unwrap();
            update_cb.emit(gi);
        });

        let i = Ingame {
            game,
            game_info: None,
            command: String::new(),

            eventsrc,
            _msg_listener,
        };
        i
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Refresh(info) => {
                self.game_info = Some(info);
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
            <div>
                <ContextProvider<Commander> context={Commander { game: self.game.clone() }}>
                    {self.game_info.clone().map(|g| html! { <GameUi gamestate={g} /> }).into_iter().collect::<Html>()}
                </ContextProvider<Commander>>
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
                <pre>
                    {serde_json::to_string_pretty(&self.game_info).unwrap()}
                </pre>
            </div>
        }
    }

    fn destroy(&mut self, _: &Context<Self>) {
        self.eventsrc.close();
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

mod utils;
pub use utils::*;

mod pregame;
mod turnstart;
mod trading;
mod trade_trigger;
mod donation;
mod attacking;

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
                PerspectiveTurnState::DonatingItem { donor } if donor == &me.player => html! { <donation::ItemDonation /> },
                PerspectiveTurnState::DonatingItem { donor } => html! { {format!("Waiting for {:?} to donate an item ...", donor)} },
                PerspectiveTurnState::TurnStart { player } if player == &me.player => html! { <turnstart::MyTurnStart /> },
                PerspectiveTurnState::TurnStart { player } => html! { {format!("Waiting for {} ...", player)} },
                PerspectiveTurnState::GameOver { winner } => html! { <div class="victory-text">{format!("The {:?} is victorious!", winner)}</div> },
                &PerspectiveTurnState::TradePending { offerer, target, item } if target == me.player => html! { <trading::TradeOffer you={p.you.clone()} {offerer} item={item.unwrap()} stack_empty={p.item_stack == 0} /> },
                PerspectiveTurnState::TradePending { offerer, target, .. } => html! { <p class="trade-text">{format!("{} is offering an item to {} ...", offerer, target)}</p> },
                &PerspectiveTurnState::ResolvingTradeTrigger { offerer, target, ref trigger, is_first_item } => html! { <trade_trigger::TradeTrigger {is_first_item} {offerer} {target} trigger={trigger.clone()} /> },

                &PerspectiveTurnState::Attacking { attacker, defender, ref state } => html! { <attacking::Attacking {attacker} {defender} myself={me.player} state={state.clone()} /> },

                &PerspectiveTurnState::DoingClairvoyant { player, .. } if player != me.player => html! { <p>{format!("Waiting for the Clairvoyant ({}) to do their work ...", player)}</p> },
                PerspectiveTurnState::DoingClairvoyant { player, item_stack } => html! { {"todo"} },
                &PerspectiveTurnState::UnsuccessfulDiplomat { diplomat, target, .. } if diplomat != me.player => html! { <p>{format!("Waiting for the Diplomat ({}) to confirm that {} does not have the requested item ...", diplomat, target)}</p> },
                PerspectiveTurnState::UnsuccessfulDiplomat { target, inventory, .. } => html! { <><p>{format!("Since {} does not have the requested item, you may see their inventory: {:?}", target, inventory)}</p><DoneLookingBtn /></> },
            };
            html! {
                <div class="hud">
                    <ContextProvider<Rc<Perspective>> context={Rc::new(p.clone())}>
                        {body}
                    </ContextProvider<Rc<Perspective>>>
                </div>
            }
        }
    }
}

