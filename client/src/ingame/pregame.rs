use super::{Commander, SimpleDropdown, Lang};
use yew::prelude::*;

use web_protocol::{Player, GameCommand};

#[derive(Properties, PartialEq)]
pub struct WaitingForPlayersProps {
    pub players: Vec<Player>,
    pub you: Option<Player>,
}

#[function_component(WaitingForPlayers)]
pub fn waiting_for_players(WaitingForPlayersProps { players, you }: &WaitingForPlayersProps) -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();
    let cmd = use_context::<Commander>().unwrap();
    let cmd2 = cmd.clone();
    html! {
        <div class="content">
            {lang.players_label()}
            <ul>
                {for players.iter().map(|p| html! { <li key={p.to_string()}>{if Some(p) == you.as_ref() { format!("{} {}", p, lang.you_label()) } else { p.to_string() }}</li> })}
            </ul>
            {match you {
                None => html! { <PlayerSelection players={players.clone()} /> },
                Some(_) => html! {
                    <>
                        <button class="button" onclick={Callback::from(move |_| cmd.cmd(GameCommand::LeaveGame))}>{lang.leave()}</button>
                        if players.len() >= 3 {
                            <button class="button" onclick={Callback::from(move |_| cmd2.cmd(GameCommand::StartGame))}>{lang.start_game()}</button>
                        }
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
    let lang = use_context::<Lang>().unwrap_or_default();
    let avail_players = Player::all().filter(|p| !props.players.contains(&p));
    let selected_join_player = use_state(|| avail_players.clone().next().unwrap());
    let selected_join_player2 = selected_join_player.clone();

    let cmd = use_context::<Commander>().unwrap();

    html! {
        <>
            <SimpleDropdown<Player> options={avail_players.collect::<Vec<_>>()} on_change={Callback::from(move |x| {
                selected_join_player2.set(x);
            })} />
            <button class="button" onclick={Callback::from(move |_| cmd.cmd(GameCommand::JoinGame(*selected_join_player)))}>{lang.join()}</button>
        </>
    }
}
