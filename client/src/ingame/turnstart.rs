use std::borrow::Cow;
use std::rc::Rc;

use web_protocol::{Command, Item, Job, Perspective, Player};
use yew::prelude::*;

use crate::ingame::playerlist::PlayerList;
use crate::ingame::{CommandButton, SimpleDropdown};

#[derive(Clone, Copy, PartialEq, Eq)]
enum WipMoveKind {
    None,
    Pass,
    AnnounceVictory,
    OfferTrade,
    Attack,

    UseClairvoyant,
    UseDiplomat,
}

#[derive(Properties, PartialEq)]
pub struct MyTurnStartProps {
    pub is_turn_end: bool,
    pub my_job: Job,
    pub job_used: bool,
}
#[function_component(MyTurnStart)]
pub fn my_turn_start(MyTurnStartProps { is_turn_end, my_job, job_used }: &MyTurnStartProps) -> Html {
    enum HasPlayer { No, One, Many }
    let perspective = use_context::<Rc<Perspective>>().unwrap();
    let movekind = use_state(|| WipMoveKind::None);
    let players = use_state(|| Vec::<Player>::new());
    let item = use_state(|| None);
    let item_idx = use_state(|| None);
    let diplomat_item = use_state(|| None);
    let action_btn = |kind: WipMoveKind, text: &'static str, has_player: HasPlayer, has_item: bool| -> Html {
        let movekind = movekind.clone();
        let players = players.clone();
        let item = item.clone();
        let item_idx = item_idx.clone();
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
                    item_idx.set(None);
                }
            }
        })}>{text}</button> }
    };

    let mut buttons = Vec::new();
    if *is_turn_end {
        buttons.push(html! { <CommandButton text={"End Turn"} command={Command::Pass} class={"actionchoice endturn"} /> });
    } else {
        buttons.extend([
            action_btn(WipMoveKind::Attack, "Attack", HasPlayer::One, false),
            action_btn(WipMoveKind::OfferTrade, "Offer Trade", HasPlayer::One, true),
            action_btn(WipMoveKind::AnnounceVictory, "Announce Victory", HasPlayer::Many, false),
            action_btn(WipMoveKind::Pass, "Pass", HasPlayer::No, false),
        ]);
    };
    if *my_job == Job::Clairvoyant && !job_used {
        buttons.push(action_btn(WipMoveKind::UseClairvoyant, "Use Clairvoyant", HasPlayer::No, false));
    }
    if *my_job == Job::Diplomat && !job_used {
        buttons.push(action_btn(WipMoveKind::UseDiplomat, "Use Diplomat", HasPlayer::One, true));
    }

    let upcoming_command = (|| Some(match (*movekind, &*players, *item, *diplomat_item) {
        (WipMoveKind::Pass, _, _, _) => Command::Pass,
        (WipMoveKind::AnnounceVictory, players, _, _) => Command::AnnounceVictory { teammates: players.clone() },
        (WipMoveKind::OfferTrade, players, Some(item), _) if players.len() == 1 => Command::OfferTrade { target: players[0], item },
        (WipMoveKind::Attack, players, _, _) if players.len() == 1 => Command::InitiateAttack { player: players[0] },
        (WipMoveKind::UseClairvoyant, _, _, _) if players.len() == 0 => Command::UseClairvoyant,
        (WipMoveKind::UseDiplomat, players, Some(return_item), Some(item)) if players.len() == 1 => Command::UseDiplomat { target: players[0], item, return_item },
        _ => return None,
    }))();

    let actiontext = match *movekind {
        WipMoveKind::None => Cow::from(""),
        WipMoveKind::Pass => Cow::from("You are going to pass."),
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
            Cow::from(format!("You are going to announce the victory of the {:?} {}.", perspective.you.faction, text))
        }
        WipMoveKind::OfferTrade => Cow::from(format!("You offer to trade a {} to {}.", item.map(|x| x.to_string()).unwrap_or("?".to_owned()), players.get(0).map(|p| p.to_string()).unwrap_or("?".to_owned()))),
        WipMoveKind::Attack => Cow::from(format!("You attack {}.", players.get(0).map(|p| p.to_string()).unwrap_or("?".to_owned()))),
        WipMoveKind::UseClairvoyant => Cow::from("You are going to use your job ability (Clairvoyant)."),
        WipMoveKind::UseDiplomat => Cow::from(format!(
            "You are going to use your job ability (Diplomat). You are demanding a {} from {} in exchange for a {}.",
            diplomat_item.map(|x| x.to_string()).unwrap_or("?".to_owned()),
            players.get(0).map(|p| p.to_string()).unwrap_or("?".to_owned()),
            item.map(|x| x.to_string()).unwrap_or("?".to_owned())
        )),
    };

    html! {
        <>
            <PlayerList selected={Some(players.clone())} block_select={matches!(*movekind, WipMoveKind::Pass | WipMoveKind::UseClairvoyant)} />
            {"Your items:"}
            <div class="itemlist">
                {for perspective.you.items.iter().enumerate().map(|(idx, &i)| {
                    let is_selected = *item_idx == Some(idx);
                    let selected = if is_selected { Some("selected") } else { None };
                    let can_select = match *movekind { WipMoveKind::OfferTrade | WipMoveKind::UseDiplomat | WipMoveKind::None => true, _ => false };
                    let selectable = if can_select { Some("selectable") } else { None };
                    let item = item.clone();
                    let item_idx = item_idx.clone();
                    html! { <div class={classes!("entry", selected, selectable)} onclick={Callback::from(move |_| {
                        if can_select {
                            if is_selected {
                                item.set(None);
                                item_idx.set(None);
                            } else {
                                item.set(Some(i));
                                item_idx.set(Some(idx));
                            }
                        }
                    })}>{format!("{:?}", i)}</div> }
                })}
            </div>
            if *movekind == WipMoveKind::UseDiplomat {
                <div class="choose-diplomat">
                    {"Ask for: "}<SimpleDropdown<Item> options={DIPLOMAT_ITEM_LIST.to_vec()} on_change={Callback::from(move |x| diplomat_item.set(Some(x)))} />
                </div>
            }
            <div class="actionlist">
                {buttons}
            </div>

            <p class="actiontext">{actiontext}</p>

            <CommandButton text={"Submit"} command={upcoming_command} class={"actionsubmit"} onclick={Callback::from(move |_| {
                movekind.set(WipMoveKind::None);
                players.set(Vec::new());
                item.set(None);
                item_idx.set(None);
            })} />
        </>
    }
}

const DIPLOMAT_ITEM_LIST: &[Item] = &[
    Item::Key,
    Item::Goblet,
    Item::BlackPearl,
    Item::Dagger,
    Item::Gloves,
    Item::PoisonRing,
    Item::CastingKnives,
    Item::Whip,
    Item::Priviledge,
    Item::Monocle,
    Item::BrokenMirror,
    Item::Sextant,
    Item::Coat,
    Item::Tome,
    Item::CoatOfArmorOfTheLoge
];