use gloo_console::log;
use gloo_timers::future::TimeoutFuture;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use web_protocol::{GameInfo, Player, GameCommand};

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

#[derive(Properties, PartialEq)]
struct GameUiProps {
    pub gamestate: GameInfo,
}

#[function_component(GameUi)]
fn game_ui(props: &GameUiProps) -> Html {
    let cmd = use_context::<Commander>().unwrap();
    match &props.gamestate {
        GameInfo::WaitingForPlayers { players, you } => html! {
            <div class="content">
                {"Players:"}
                <ul>
                    {for players.iter().map(|p| html! { <li key={p.to_string()}>{if Some(p) == you.as_ref() { format!("{} (you)", p) } else { p.to_string() }}</li> })}
                </ul>
                {match you {
                    None => html! { <PlayerSelection players={players.clone()} /> },
                    Some(_) => html! {
                        <button class="button" onclick={Callback::once(move |_| cmd.cmd(GameCommand::LeaveGame))}>{"Leave"}</button>
                    },
                }}
            </div>
        },
        GameInfo::Game(_) => html! {},
    }
}

#[derive(Properties, PartialEq)]
struct PlayerSelectionProps {
    pub players: Vec<Player>,
}

#[function_component(PlayerSelection)]
fn player_selection(props: &PlayerSelectionProps) -> Html {
    let avail_players = Player::all().filter(|p| !props.players.contains(&p));
    let selected_join_player = use_state(|| avail_players.clone().next().unwrap());
    let selected_join_player2 = selected_join_player.clone();

    let cmd = use_context::<Commander>().unwrap();

    html! {
        <>
            <div class="select">
                <select onchange={Callback::from(move |e: Event| {
                    let p: Player = e.target_unchecked_into::<HtmlSelectElement>().value().parse().unwrap();
                    selected_join_player2.set(p);
                })}>
                    {for avail_players.map(|p| html! { <option value={p.to_string()} selected={p == *selected_join_player}>{p.to_string()}</option> })}
                </select>
            </div>
            <button class="button" onclick={Callback::once(move |_| cmd.cmd(GameCommand::JoinGame(*selected_join_player)))}>{"Join"}</button>
        </>
    }
}
