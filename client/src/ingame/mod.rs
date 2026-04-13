use std::rc::Rc;

use gloo_console::log;
use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{EventSource, HtmlInputElement, MessageEvent};
use yew::prelude::*;
use yew_router::hooks::use_location;
use web_protocol::{GameCommand, GameInfo, Item, Perspective, Player, PerspectiveTurnState, WinningFaction, Faction};

pub use crate::i18n::{Lang, Translate, faction_name, action_log_text};

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

        Ingame { game, game_info: None, eventsrc, _msg_listener }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Refresh(info) => { self.game_info = Some(info); }
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
mod myfaction;
mod actionlog;
mod clairvoyant;
mod spectating;

#[derive(Properties, PartialEq)]
struct GameUiProps {
    pub gamestate: GameInfo,
}

#[function_component(GameUi)]
fn game_ui(props: &GameUiProps) -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();

    // Hoisted selection state — shared between top-left PlayerList/top-right ItemList and the body component
    let player_selection = use_state(|| Vec::<Player>::new());
    let item_selection = itemlist::ItemWithIndex::use_new();
    let block_players = use_state(|| false);

    match &props.gamestate {
        GameInfo::WaitingForPlayers { players, you } => html! {
            <pregame::WaitingForPlayers players={players.clone()} you={you.clone()} />
        },
        GameInfo::Game(p) => {
            let me = &p.players[p.your_player_index];

            // Compute item blocklist for top-right ItemList based on turn state
            let item_blocklist: Vec<Item> = match &p.turn {
                &PerspectiveTurnState::TradePending { item: Some(item), target, .. } if target == me.player => {
                    let combo = vec![Item::BagGoblet, Item::BagKey];
                    if combo.contains(&item) { combo } else { vec![] }
                }
                _ => vec![],
            };

            let body = match &p.turn {
                PerspectiveTurnState::DonatingItem { donor } if donor == &me.player => html! {
                    <donation::ItemDonation
                        players={player_selection.clone()}
                        item={item_selection.clone()}
                    />
                },
                PerspectiveTurnState::DonatingItem { donor } => html! {
                    <p>{lang.waiting_for_donate(&donor.to_string())}</p>
                },
                PerspectiveTurnState::TurnStart { player } if player == &me.player => html! {
                    <turnstart::MyTurnStart
                        players={player_selection.clone()}
                        item={item_selection.clone()}
                        block_players={block_players.clone()}
                        my_job={p.you.job}
                        job_used={p.you.job_is_visible}
                        is_turn_end={false}
                    />
                },
                PerspectiveTurnState::TurnStart { player } => html! {
                    <p>{lang.waiting_for(&player.to_string())}</p>
                },
                PerspectiveTurnState::TurnEndPhase { player } if player == &me.player => html! {
                    <turnstart::MyTurnStart
                        players={player_selection.clone()}
                        item={item_selection.clone()}
                        block_players={block_players.clone()}
                        my_job={p.you.job}
                        job_used={p.you.job_is_visible}
                        is_turn_end={true}
                    />
                },
                PerspectiveTurnState::TurnEndPhase { player } => html! {
                    <p>{lang.waiting_for(&player.to_string())}</p>
                },
                PerspectiveTurnState::GameOver { winner: WinningFaction::Normal(Faction::Order) } => html! {
                    <div class="victory-screen victory-order">
                        <p class="victory-text">{lang.victory_order()}</p>
                    </div>
                },
                PerspectiveTurnState::GameOver { winner: WinningFaction::Normal(Faction::Brotherhood) } => html! {
                    <div class="victory-screen victory-brotherhood">
                        <p class="victory-text">{lang.victory_brotherhood()}</p>
                    </div>
                },
                PerspectiveTurnState::GameOver { winner: WinningFaction::Traitor(traitor) } => html! {
                    <div class="victory-screen victory-loge">
                        <p class="victory-text">{lang.victory_traitor(&traitor.to_string())}</p>
                    </div>
                },
                &PerspectiveTurnState::TradePending { offerer, target, item } if target == me.player => html! {
                    <trading::TradeOffer
                        you={p.you.clone()}
                        {offerer}
                        item={item.unwrap()}
                        stack_empty={p.item_stack == 0}
                        item_selection={item_selection.clone()}
                    />
                },
                PerspectiveTurnState::TradePending { offerer, target, .. } => html! {
                    <p class="trade-text">{lang.waiting_for_trade(&offerer.to_string(), &target.to_string())}</p>
                },
                &PerspectiveTurnState::ResolvingTradeTrigger { giver, receiver, ref trigger } => html! {
                    <trade_trigger::TradeTrigger myself={me.player} {giver} {receiver} trigger={trigger.clone()} />
                },
                &PerspectiveTurnState::Attacking { attacker, defender, ref state } => html! {
                    <attacking::Attacking {attacker} {defender} myself={me.player} state={state.clone()} />
                },
                &PerspectiveTurnState::DoingClairvoyant { player, .. } if player != me.player => html! {
                    <p>{lang.waiting_for_clairvoyant(&player.to_string())}</p>
                },
                PerspectiveTurnState::DoingClairvoyant { player: _, item_stack } => html! {
                    <clairvoyant::Clairvoyant item_stack={item_stack.clone().unwrap()} />
                },
                &PerspectiveTurnState::UnsuccessfulDiplomat { diplomat, target, .. } if diplomat != me.player => html! {
                    <p>{lang.waiting_for_diplomat(&diplomat.to_string(), &target.to_string())}</p>
                },
                PerspectiveTurnState::UnsuccessfulDiplomat { target, inventory, .. } => {
                    let items_str = inventory.iter().flatten().map(|x| x.tr_name(lang)).collect::<Vec<_>>().join(", ");
                    html! { <><p>{lang.diplomat_no_item(&target.to_string(), &items_str)}</p><DoneLookingBtn /></> }
                },
            };

            html! {
                <div class="hud">
                    <ContextProvider<Rc<Perspective>> context={Rc::new(p.clone())}>
                        <div class="hud-topleft">
                            <playerlist::PlayerList
                                selected={Some(player_selection.clone())}
                                block_select={*block_players}
                            />
                        </div>
                        <div class="hud-topright">
                            <myfaction::MyFaction />
                            <myjob::MyJob />
                            <itemlist::ItemList
                                selection={Some(item_selection.clone())}
                                blocklist={item_blocklist}
                            />
                        </div>
                        <div class="hud-bottomleft">
                            <actionlog::ActionLog />
                        </div>
                        <div class="hud-bottomright">
                            {body}
                        </div>
                    </ContextProvider<Rc<Perspective>>>
                </div>
            }
        }
        GameInfo::Spectating(s) => html! { <spectating::Spectating state={s.clone()} /> },
    }
}
