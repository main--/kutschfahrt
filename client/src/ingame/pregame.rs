use super::{Commander, SimpleDropdown};
use yew::prelude::*;

use web_protocol::{Player, GameCommand};

#[derive(Properties, PartialEq)]
pub struct WaitingForPlayersProps {
    pub players: Vec<Player>,
    pub you: Option<Player>,
}

#[function_component(WaitingForPlayers)]
pub fn waiting_for_players(props: &WaitingForPlayersProps) -> Html {
    let WaitingForPlayersProps { players, you } = props.clone();
    let cmd = use_context::<Commander>().unwrap();
    let cmd2 = cmd.clone();
    html! {
        <div class="content">
            {"Players:"}
            <ul>
                {for players.iter().map(|p| html! { <li key={p.to_string()}>{if Some(p) == you.as_ref() { format!("{} (you)", p) } else { p.to_string() }}</li> })}
            </ul>
            {match you {
                None => html! { <PlayerSelection players={players.clone()} /> },
                Some(_) => html! {
                    <>
                        <button class="button" onclick={Callback::once(move |_| cmd.cmd(GameCommand::LeaveGame))}>{"Leave"}</button>
                        <button class="button" onclick={Callback::once(move |_| cmd2.cmd(GameCommand::StartGame))}>{"Start Game"}</button>
                    </>
                },
            }}
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct PlayerSelectionProps {
    pub players: Vec<Player>,
}

#[function_component(PlayerSelection)]
pub fn player_selection(props: &PlayerSelectionProps) -> Html {
    let avail_players = Player::all().filter(|p| !props.players.contains(&p));
    let selected_join_player = use_state(|| avail_players.clone().next().unwrap());
    let selected_join_player2 = selected_join_player.clone();

    let cmd = use_context::<Commander>().unwrap();

    html! {
        <>
            <SimpleDropdown<Player> options={avail_players.collect::<Vec<_>>()} on_change={Callback::from(move |x| {
                selected_join_player2.set(x);
            })} />
            <button class="button" onclick={Callback::once(move |_| cmd.cmd(GameCommand::JoinGame(*selected_join_player)))}>{"Join"}</button>
        </>
    }
}

