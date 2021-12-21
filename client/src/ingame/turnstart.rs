use std::borrow::Cow;

use web_protocol::{Perspective, GameCommand, Command, Player};
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

    let actiontext = match *movekind {
        WipMoveKind::None => Cow::from(""),
        WipMoveKind::Pass => Cow::from("You pass."),
        WipMoveKind::AnnounceVictory => {
            let mut text = String::new();
            if players.len() == 0 {
                text += "alone";
            } else {
                text += "together with ";
                for (i, p) in players.iter().enumerate() {
                    if i == 0 {
                    } else if i == players.len() - 1 {
                        text += " and ";
                    } else {
                        text += ", ";
                    }
                    text += &p.to_string();
                }
            }
            Cow::from(format!("You are going to announce the victory of the {:?} {}.", props.perspective.you.faction, text))
        }
        WipMoveKind::OfferTrade => Cow::from(format!("You offer to trade a {} to {}.", item.map(|i| format!("{:?}", i)).unwrap_or("?".to_owned()), players.get(0).map(|p| p.to_string()).unwrap_or("?".to_owned()))),
        WipMoveKind::Attack => Cow::from(format!("You attack {}.", players.get(0).map(|p| p.to_string()).unwrap_or("?".to_owned()))),
    };

    html! {
        <>
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

            <p class="actiontext">{actiontext}</p>

            <button class="button actionsubmit" disabled={upcoming_command.is_none()} onclick={Callback::once(move |_| {
                match upcoming_command {
                    Some(c) => cmd.cmd(GameCommand::Command(c)),
                    None => (),
                }
            })}>{"Submit"}</button>
        </>
    }
}
