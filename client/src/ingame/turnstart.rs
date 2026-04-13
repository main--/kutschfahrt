use std::borrow::Cow;
use std::rc::Rc;

use web_protocol::{Command, Item, Job, Perspective, Player, VictoryFlavor};
use yew::prelude::*;

use crate::ingame::itemlist::{ItemList, ItemWithIndex};
use crate::ingame::myfaction::MyFaction;
use crate::ingame::myjob::MyJob;
use crate::ingame::playerlist::PlayerList;
use crate::ingame::{CommandButton, SimpleDropdown, Lang, Translate, faction_name};

#[derive(Clone, Copy, PartialEq, Eq)]
enum WipMoveKind {
    None,
    Pass,
    AnnounceVictory,
    LogeVictory,
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
    let lang = use_context::<Lang>().unwrap_or_default();
    let movekind = use_state(|| WipMoveKind::None);
    let players = use_state(|| Vec::<Player>::new());
    let item = ItemWithIndex::use_new();
    let diplomat_item = use_state(|| DIPLOMAT_ITEM_LIST[0]);
    let confirming_victory = use_state(|| false);

    let action_btn = |kind: WipMoveKind, text: AttrValue, has_player: HasPlayer, has_item: bool| -> Html {
        let movekind = movekind.clone();
        let players = players.clone();
        let item = item.clone();
        let confirming_victory = confirming_victory.clone();
        let active = if *movekind == kind { Some("is-dark") } else { None };
        html! { <button class={classes!("button", "actionchoice", active)} onclick={Callback::from(move |_| {
            if *movekind == kind {
                movekind.set(WipMoveKind::None);
            } else {
                movekind.set(kind);
                confirming_victory.set(false);
                match has_player {
                    HasPlayer::No => players.set(Vec::new()),
                    HasPlayer::One => players.set(players.get(0).cloned().into_iter().collect()),
                    HasPlayer::Many => (),
                }
                if !has_item {
                    item.reset();
                }
            }
        })}>{text}</button> }
    };

    let mut buttons = Vec::new();
    if *is_turn_end {
        buttons.push(html! { <CommandButton text={lang.end_turn()} command={Command::Pass} class={"actionchoice endturn"} /> });
    } else {
        buttons.extend([
            action_btn(WipMoveKind::Attack, lang.attack().into(), HasPlayer::One, false),
            action_btn(WipMoveKind::OfferTrade, lang.offer_trade().into(), HasPlayer::One, true),
            action_btn(WipMoveKind::AnnounceVictory, lang.announce_victory().into(), HasPlayer::Many, false),
            action_btn(WipMoveKind::Pass, lang.pass().into(), HasPlayer::No, false),
        ]);
    };
    if *my_job == Job::Clairvoyant && !job_used {
        buttons.push(action_btn(WipMoveKind::UseClairvoyant, lang.use_clairvoyant().into(), HasPlayer::No, false));
    }
    if *my_job == Job::Diplomat && !job_used {
        buttons.push(action_btn(WipMoveKind::UseDiplomat, lang.use_diplomat().into(), HasPlayer::One, true));
    }

    let is_victory_item = |&&x: &&Item| x == Item::Key || x == Item::Goblet || ((perspective.item_stack == 0) && (x == Item::BagKey || x == Item::BagGoblet));
    if perspective.you.items.contains(&Item::CoatOfArmorOfTheLoge) && perspective.you.items.iter().filter(is_victory_item).count() >= 3 {
        buttons.push(action_btn(WipMoveKind::LogeVictory, lang.loge_victory().into(), HasPlayer::No, false));
    }

    let upcoming_command = (|| Some(match (*movekind, &*players, item.item(), *diplomat_item) {
        (WipMoveKind::Pass, _, _, _) => Command::Pass,
        (WipMoveKind::LogeVictory, _, _, _) => Command::AnnounceVictory { flavor: VictoryFlavor::Loge },
        (WipMoveKind::AnnounceVictory, players, _, _) => Command::AnnounceVictory { flavor: VictoryFlavor::Normal { teammates: players.clone() } },
        (WipMoveKind::OfferTrade, players, Some(item), _) if players.len() == 1 => Command::OfferTrade { target: players[0], item },
        (WipMoveKind::Attack, players, _, _) if players.len() == 1 => Command::InitiateAttack { player: players[0] },
        (WipMoveKind::UseClairvoyant, _, _, _) if players.len() == 0 => Command::UseClairvoyant,
        (WipMoveKind::UseDiplomat, players, Some(return_item), item) if players.len() == 1 => Command::UseDiplomat { target: players[0], item, return_item },
        _ => return None,
    }))();

    let is_victory = matches!(&upcoming_command, Some(Command::AnnounceVictory { .. }));

    let actiontext: Cow<str> = match *movekind {
        WipMoveKind::None => Cow::from(""),
        WipMoveKind::Pass => Cow::from(lang.will_pass()),
        WipMoveKind::LogeVictory => Cow::from(lang.will_loge_victory()),
        WipMoveKind::AnnounceVictory => {
            let faction = perspective.you.effective_faction();
            let faction_str = faction_name(faction, lang);
            let allies = if players.len() == 0 {
                lang.alone_word().to_owned()
            } else {
                let mut text = lang.together_with_word().to_owned() + " ";
                for (i, p) in players.iter().enumerate() {
                    if i == 0 {
                    } else if i == players.len() - 1 {
                        text += &format!(" {} ", lang.and_word());
                    } else {
                        text += ", ";
                    }
                    text += &p.to_string();
                }
                text
            };
            Cow::from(lang.will_announce_victory(faction_str, &allies))
        }
        WipMoveKind::OfferTrade => Cow::from(lang.will_offer_trade(
            item.item().map(|x| x.tr_name(lang)).unwrap_or("?"),
            &players.get(0).map(|p| p.to_string()).unwrap_or("?".to_owned()),
        )),
        WipMoveKind::Attack => Cow::from(lang.will_attack(&players.get(0).map(|p| p.to_string()).unwrap_or("?".to_owned()))),
        WipMoveKind::UseClairvoyant => Cow::from(lang.will_clairvoyant()),
        WipMoveKind::UseDiplomat => Cow::from(lang.will_diplomat(
            diplomat_item.tr_name(lang),
            &players.get(0).map(|p| p.to_string()).unwrap_or("?".to_owned()),
            item.item().map(|x| x.tr_name(lang)).unwrap_or("?"),
        )),
    };

    html! {
        <>
            <PlayerList selected={Some(players.clone())} block_select={matches!(*movekind, WipMoveKind::Pass | WipMoveKind::UseClairvoyant)} />
            <MyFaction />
            <MyJob />
            <ItemList selection={match *movekind { WipMoveKind::OfferTrade | WipMoveKind::UseDiplomat | WipMoveKind::None => Some(item.clone()), _ => None }} />

            if *movekind == WipMoveKind::UseDiplomat {
                <div class="choose-diplomat">
                    {lang.ask_for()}{" "}<SimpleDropdown<Item> options={DIPLOMAT_ITEM_LIST.to_vec()} on_change={Callback::from(move |x| diplomat_item.set(x))} />
                </div>
            }
            <div class="actionlist">
                {buttons}
            </div>

            <p class="actiontext">{actiontext}</p>

            if is_victory && *confirming_victory {
                <div class="victory-confirm">
                    <p>{lang.victory_confirm()}</p>
                    <CommandButton text={lang.confirm_yes()} command={upcoming_command} class={"is-success"} onclick={Callback::from({
                        let movekind = movekind.clone();
                        let players = players.clone();
                        let item = item.clone();
                        let confirming_victory = confirming_victory.clone();
                        move |_| {
                            movekind.set(WipMoveKind::None);
                            players.set(Vec::new());
                            item.reset();
                            confirming_victory.set(false);
                        }
                    })} />
                    <button class="button is-light" onclick={Callback::from({
                        let confirming_victory = confirming_victory.clone();
                        move |_| confirming_victory.set(false)
                    })}>{lang.confirm_no()}</button>
                </div>
            } else if is_victory {
                <button class="button actionsubmit is-warning" onclick={Callback::from(move |_| confirming_victory.set(true))}>{lang.submit()}</button>
            } else {
                <CommandButton text={lang.submit()} command={upcoming_command} class={"actionsubmit"} onclick={Callback::from({
                    let movekind = movekind.clone();
                    let players = players.clone();
                    let item = item.clone();
                    move |_| {
                        movekind.set(WipMoveKind::None);
                        players.set(Vec::new());
                        item.reset();
                    }
                })} />
            }
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
