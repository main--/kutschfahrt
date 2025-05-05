use std::rc::Rc;

use gloo_console::log;
use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{EventSource, HtmlInputElement, MessageEvent};
use yew::prelude::*;
use yew_router::hooks::use_location;
use web_protocol::{GameCommand, GameInfo, Perspective, PerspectiveTurnState, WinningFaction};

pub struct Ingame {
    game: String,
    game_info: Option<GameInfo>,

    eventsrc: EventSource,
    _msg_listener: EventListener,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub game: String,
}

pub enum Msg {
    Refresh(GameInfo),
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
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <ContextProvider<Commander> context={Commander { game: self.game.clone() }}>
                    {self.game_info.clone().map(|g| html! { <GameUi gamestate={g} /> }).into_iter().collect::<Html>()}
                </ContextProvider<Commander>>
                <DevMode game_id={self.game.clone()} game_info={self.game_info.clone()} />
            </div>
        }
    }

    fn destroy(&mut self, _: &Context<Self>) {
        self.eventsrc.close();
    }
}

#[derive(Properties, PartialEq)]
struct DevModeProps {
    game_id: String,
    game_info: Option<GameInfo>,
}
#[function_component(DevMode)]
fn dev_mode(DevModeProps { game_id, game_info }: &DevModeProps) -> Html {
    let location = use_location();
    let command = use_state_eq(|| String::new());
    let oninput = {
        let command = command.clone();
        Callback::from(move |e: InputEvent| { let input: HtmlInputElement = e.target_unchecked_into(); command.set(input.value()); })
    };

    if !location.map_or(false, |x| x.hash() == "#dev") {
        return html! {};
    }

    let game_id = game_id.clone();
    html! {
        <>
            <input
                value={(*command).clone()}
                oninput={oninput}
                onkeypress={Callback::from(move |e: KeyboardEvent| {
                    if e.key() == "Enter" {
                        let path = format!("/api/game/{}", game_id);
                        if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&*command) {
                            command.set(String::new());
                            wasm_bindgen_futures::spawn_local(async move {
                                super::post_json(&path, &cmd).await;
                            });
                        }
                    }
                })}
            />
            <pre>
                {serde_json::to_string_pretty(&game_info).unwrap()}
            </pre>
        </>
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
mod playerlist;
mod itemlist;
mod myjob;
mod actionlog;
mod clairvoyant;

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
            let mut hide_playerlist = false;
            let body = match &p.turn {
                PerspectiveTurnState::DonatingItem { donor } if donor == &me.player => html! { <donation::ItemDonation /> },
                PerspectiveTurnState::DonatingItem { donor } => html! { {format!("Waiting for {:?} to donate an item ...", donor)} },
                PerspectiveTurnState::TurnStart { player } if player == &me.player => {
                    hide_playerlist = true;
                    html! { <turnstart::MyTurnStart my_job={p.you.job} job_used={p.you.job_is_visible} is_turn_end={false} /> }
                },
                PerspectiveTurnState::TurnStart { player } => html! { {format!("Waiting for {} ...", player)} },
                PerspectiveTurnState::TurnEndPhase { player } if player == &me.player => {
                    hide_playerlist = true;
                    html! { <turnstart::MyTurnStart my_job={p.you.job} job_used={p.you.job_is_visible} is_turn_end={true} /> }
                },
                PerspectiveTurnState::TurnEndPhase { player } => html! { {format!("Waiting for {} to end their turn ...", player)} },
                PerspectiveTurnState::GameOver { winner: WinningFaction::Normal(winner) } => html! { <div class="victory-text">{format!("The {:?} is victorious!", winner)}</div> },
                PerspectiveTurnState::GameOver { winner: WinningFaction::Traitor(traitor) } => html! { <div class="victory-text">{format!("The sole victor is {traitor}!")}</div> },
                &PerspectiveTurnState::TradePending { offerer, target, item } if target == me.player => html! { <trading::TradeOffer you={p.you.clone()} {offerer} item={item.unwrap()} stack_empty={p.item_stack == 0} /> },
                PerspectiveTurnState::TradePending { offerer, target, .. } => html! { <p class="trade-text">{format!("{} is offering an item to {} ...", offerer, target)}</p> },
                &PerspectiveTurnState::ResolvingTradeTrigger { giver, receiver, ref trigger } => html! { <trade_trigger::TradeTrigger myself={me.player} {giver} {receiver} trigger={trigger.clone()} /> },

                &PerspectiveTurnState::Attacking { attacker, defender, ref state } => html! { <attacking::Attacking {attacker} {defender} myself={me.player} state={state.clone()} /> },

                &PerspectiveTurnState::DoingClairvoyant { player, .. } if player != me.player => html! { <p>{format!("Waiting for the Clairvoyant ({}) to do their work ...", player)}</p> },
                PerspectiveTurnState::DoingClairvoyant { player: _, item_stack } => html! { <clairvoyant::Clairvoyant item_stack={item_stack.clone().unwrap()} /> },
                &PerspectiveTurnState::UnsuccessfulDiplomat { diplomat, target, .. } if diplomat != me.player => html! { <p>{format!("Waiting for the Diplomat ({}) to confirm that {} does not have the requested item ...", diplomat, target)}</p> },
                PerspectiveTurnState::UnsuccessfulDiplomat { target, inventory, .. } => html! { <><p>{format!("Since {} does not have the requested item, you may see their inventory: {:?}", target, inventory)}</p><DoneLookingBtn /></> },
            };
            html! {
                <div class="hud">
                    <ContextProvider<Rc<Perspective>> context={Rc::new(p.clone())}>
                        if !hide_playerlist {
                            <playerlist::PlayerList />
                            <myjob::MyJob />
                            <itemlist::ItemList />
                        }
                        {body}
                        <actionlog::ActionLog />
                    </ContextProvider<Rc<Perspective>>>
                </div>
            }
        }
    }
}

