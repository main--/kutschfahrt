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
mod modname {
    use web_protocol::{PerspectivePlayer, Perspective, GameCommand, Command, Player, Item};
    use yew::prelude::*;

    use crate::ingame::Commander;
    //{"Command": {"action": "pass"}}

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum WipMoveKind {
        None,
        Pass,
        AnnounceVictory,
        OfferTrade,
        Attack
    }

    #[derive(Properties, PartialEq)]
    pub struct MyTurnStartProps {
        pub perspective: Perspective,
    }
    #[function_component(MyTurnStart)]
    pub fn my_turn_start(props: &MyTurnStartProps) -> Html {
        enum HasPlayer { No, One, Many }
        let cmd = use_context::<Commander>().unwrap();
        let movekind = use_state(|| WipMoveKind::None);
        let players = use_state(|| Vec::<Player>::new());
        let item = use_state(|| None);
        let action_btn = |kind: WipMoveKind, text: &'static str, has_player: HasPlayer, has_item: bool| -> Html {
            let movekind = movekind.clone();
            let players = players.clone();
            let item = item.clone();
            let active = if *movekind == kind { Some("is-dark") } else { None };
            html! { <button class={classes!("button", "actionchoice", active)} onclick={Callback::once(move |_| {
                if *movekind == kind {
                    movekind.set(WipMoveKind::None);
                } else {
                    movekind.set(kind);
                    match has_player {
                        HasPlayer::No => players.set(Vec::new()),
                        HasPlayer::One => players.set(players.get(0).cloned().into_iter().collect()),
                        HasPlayer::Many => (),
                    }
                    if !has_item {
                        item.set(None);
                    }
                }
            })}>{text}</button> }
        };
        let attackbtn = action_btn(WipMoveKind::Attack, "Attack", HasPlayer::One, false);
        let offertradebtn = action_btn(WipMoveKind::OfferTrade, "Offer Trade", HasPlayer::One, true);
        let announcevictorybtn = action_btn(WipMoveKind::AnnounceVictory, "Announce Victory", HasPlayer::Many, false);
        let passbtn = action_btn(WipMoveKind::Pass, "Pass", HasPlayer::No, false);

        let upcoming_command = (|| Some(match (*movekind, &*players, *item) {
            (WipMoveKind::Pass, _, _) => Command::Pass,
            (WipMoveKind::AnnounceVictory, players, _) => Command::AnnounceVictory { teammates: players.clone() },
            (WipMoveKind::OfferTrade, players, Some(item)) if players.len() == 1 => Command::OfferTrade { target: players[0], item },
            (WipMoveKind::Attack, players, _) if players.len() == 1 => Command::InitiateAttack { player: players[0] },
            _ => return None,
        }))();

        html! {
            <div class="hud">
                <div class="playerlist">
                    {for props.perspective.players.iter().enumerate().map(|(i, p)| {
                        let is_you = i == props.perspective.your_player_index;
                        let you = if is_you { Some("you") } else { None };
                        let selected = if players.contains(&p.player) { Some("selected") } else { None };
                        let can_select = match *movekind { WipMoveKind::Pass => false, _ => !is_you };
                        let selectable = if can_select { Some("selectable") } else { None };

                        let players = players.clone();
                        let player = p.player;

                        html! {
                            <div class={classes!("entry", you, selected, selectable)} onclick={Callback::from(move |_| {
                                if can_select {
                                    let mut p = (*players).clone();
                                    if p.contains(&player) {
                                        p.retain(|&x| x != player);
                                    } else {
                                        p.push(player);
                                    }
                                    players.set(p);
                                }
                            })}>
                                <div class="name">{p.player.to_string()}</div>
                                <div class="job">{p.job.map(|j| format!("{:?}", j)).unwrap_or("?".to_owned())}</div>
                                <div class="item_count">{p.item_count}</div>
                            </div>
                        }
                    })}
                </div>
                <div class="itemlist">
                    {for props.perspective.you.items.iter().map(|&i| {
                        let is_selected = *item == Some(i);
                        let selected = if *item == Some(i) { Some("selected") } else { None };
                        let can_select = match *movekind { WipMoveKind::OfferTrade | WipMoveKind::None => true, _ => false };
                        let selectable = if can_select { Some("selectable") } else { None };
                        let item = item.clone();
                        html! { <div class={classes!("entry", selected, selectable)} onclick={Callback::from(move |_| {
                            if can_select {
                                if is_selected {
                                    item.set(None);
                                } else {
                                    item.set(Some(i));
                                }
                            }
                        })}>{format!("{:?}", i)}</div> }
                    })}
                </div>
                <div class="actionlist">
                    {attackbtn}
                    {offertradebtn}
                    {announcevictorybtn}
                    {passbtn}
                </div>

                <button class="button actionsubmit" disabled={upcoming_command.is_none()} onclick={Callback::once(move |_| {
                    match upcoming_command {
                        Some(c) => cmd.cmd(GameCommand::Command(c)),
                        None => (),
                    }
                })}>{"Submit"}</button>
            </div>
        }
    }
}


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
            match &p.turn {
                PerspectiveTurnState::TurnStart { player } if player == &me.player => html! { <modname::MyTurnStart perspective={p.clone()} /> },
                PerspectiveTurnState::TurnStart { player } => html! { {format!("Waiting for {} ...", player)} },
                PerspectiveTurnState::GameOver { winner } => todo!(),
                PerspectiveTurnState::TradePending { offerer, target, item } => todo!(),
                PerspectiveTurnState::ResolvingTradeTrigger { offerer, target, trigger } => todo!(),
                PerspectiveTurnState::Attacking { attacker, defender, state } => todo!(),
            }
        }
    }
}
