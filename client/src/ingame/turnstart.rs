use std::borrow::Cow;
use std::rc::Rc;

use web_protocol::{Command, Item, Job, Perspective, Player, VictoryFlavor};
use yew::prelude::*;

use crate::ingame::itemlist::ItemWithIndex;
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
    pub players: UseStateHandle<Vec<Player>>,   // used only for AnnounceVictory
    pub item: ItemWithIndex,
    pub block_players: UseStateHandle<bool>,
}

#[function_component(MyTurnStart)]
pub fn my_turn_start(props: &MyTurnStartProps) -> Html {
    enum HasItem { Yes, No }
    let perspective = use_context::<Rc<Perspective>>().unwrap();
    let lang = use_context::<Lang>().unwrap_or_default();
    let MyTurnStartProps { is_turn_end, my_job, job_used, players, item, block_players } = props;

    let myself = perspective.players[perspective.your_player_index].player;
    let other_players: Vec<Player> = perspective.players.iter()
        .map(|p| p.player)
        .filter(|&p| p != myself)
        .collect();

    let movekind = use_state(|| WipMoveKind::None);
    let target = use_state(|| other_players.first().copied());   // for Attack / Trade / Diplomat
    let diplomat_item = use_state(|| DIPLOMAT_ITEM_LIST[0]);
    let confirming_victory = use_state(|| false);

    // PlayerList is always read-only; selections are managed in the action area
    {
        let block_players = block_players.clone();
        use_effect_with((), move |_| {
            block_players.set(true);
        });
    }

    // Reset target to first available player when switching to single-target modes
    {
        let target = target.clone();
        let first = other_players.first().copied();
        use_effect_with(*movekind, move |&mk| {
            if matches!(mk, WipMoveKind::Attack | WipMoveKind::OfferTrade | WipMoveKind::UseDiplomat) {
                target.set(first);
            }
        });
    }

    let action_btn = |kind: WipMoveKind, text: AttrValue, has_item: HasItem| -> Html {
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
                // AnnounceVictory keeps existing player selections; others clear them
                if kind != WipMoveKind::AnnounceVictory {
                    players.set(Vec::new());
                }
                if matches!(has_item, HasItem::No) {
                    item.reset();
                }
            }
        })}>{text}</button> }
    };

    let has_black_pearl = perspective.you.items.contains(&Item::BlackPearl);

    let mut buttons = Vec::new();
    if *is_turn_end {
        buttons.push(html! { <CommandButton text={lang.end_turn()} command={Command::Pass} class={"actionchoice endturn"} /> });
    } else {
        buttons.extend([
            action_btn(WipMoveKind::Attack,      lang.attack().into(),      HasItem::No),
            action_btn(WipMoveKind::OfferTrade,  lang.offer_trade().into(), HasItem::Yes),
        ]);
        if has_black_pearl {
            buttons.push(html! {
                <span class="disabled-btn-wrap" data-tooltip={lang.black_pearl_no_victory()}>
                    <button class="button actionchoice" disabled={true}>
                        {lang.announce_victory()}
                    </button>
                </span>
            });
        } else {
            buttons.push(action_btn(WipMoveKind::AnnounceVictory, lang.announce_victory().into(), HasItem::No));
        }
        buttons.push(action_btn(WipMoveKind::Pass, lang.pass().into(), HasItem::No));
    };
    if *my_job == Job::Clairvoyant && !job_used {
        buttons.push(action_btn(WipMoveKind::UseClairvoyant, lang.use_clairvoyant().into(), HasItem::No));
    }
    if *my_job == Job::Diplomat && !job_used {
        buttons.push(action_btn(WipMoveKind::UseDiplomat, lang.use_diplomat().into(), HasItem::Yes));
    }

    let is_victory_item = |&&x: &&Item| x == Item::Key || x == Item::Goblet
        || ((perspective.item_stack == 0) && (x == Item::BagKey || x == Item::BagGoblet));
    if perspective.you.items.contains(&Item::CoatOfArmorOfTheLoge)
        && perspective.you.items.iter().filter(is_victory_item).count() >= 3
    {
        buttons.push(action_btn(WipMoveKind::LogeVictory, lang.loge_victory().into(), HasItem::No));
    }

    let item_val = match *movekind {
        WipMoveKind::OfferTrade | WipMoveKind::UseDiplomat => item.item(),
        _ => None,
    };

    let upcoming_command = (|| Some(match (*movekind, &**players, item_val, *diplomat_item, *target) {
        (WipMoveKind::Pass, _, _, _, _)         => Command::Pass,
        (WipMoveKind::LogeVictory, _, _, _, _)  => Command::AnnounceVictory { flavor: VictoryFlavor::Loge },
        (WipMoveKind::AnnounceVictory, p, _, _, _) =>
            Command::AnnounceVictory { flavor: VictoryFlavor::Normal { teammates: p.to_vec() } },
        (WipMoveKind::UseClairvoyant, _, _, _, _) => Command::UseClairvoyant,
        (WipMoveKind::Attack,     _, _, _, Some(t)) => Command::InitiateAttack { player: t },
        (WipMoveKind::OfferTrade, _, Some(i), _, Some(t)) => Command::OfferTrade { target: t, item: i },
        (WipMoveKind::UseDiplomat,_, Some(ret), demand, Some(t)) =>
            Command::UseDiplomat { target: t, item: demand, return_item: ret },
        _ => return None,
    }))();

    let is_victory = matches!(&upcoming_command, Some(Command::AnnounceVictory { .. }));

    let actiontext: Cow<str> = match *movekind {
        WipMoveKind::None => Cow::from(""),
        WipMoveKind::Pass => Cow::from(lang.will_pass()),
        WipMoveKind::LogeVictory => Cow::from(lang.will_loge_victory()),
        WipMoveKind::AnnounceVictory => {
            let faction_str = faction_name(perspective.you.effective_faction(), lang);
            let allies = if players.len() == 0 {
                lang.alone_word().to_owned()
            } else {
                let mut text = lang.together_with_word().to_owned() + " ";
                for (i, p) in players.iter().enumerate() {
                    if i == 0 {} else if i == players.len() - 1 {
                        text += &format!(" {} ", lang.and_word());
                    } else { text += ", "; }
                    text += &p.to_string();
                }
                text
            };
            Cow::from(lang.will_announce_victory(faction_str, &allies))
        }
        WipMoveKind::OfferTrade => Cow::from(lang.will_offer_trade(
            item.item().map(|x| x.tr_name(lang)).unwrap_or("?"),
            &target.map(|p| p.to_string()).unwrap_or("?".to_owned()),
        )),
        WipMoveKind::Attack => Cow::from(lang.will_attack(
            &target.map(|p| p.to_string()).unwrap_or("?".to_owned()),
        )),
        WipMoveKind::UseClairvoyant => Cow::from(lang.will_clairvoyant()),
        WipMoveKind::UseDiplomat => Cow::from(lang.will_diplomat(
            diplomat_item.tr_name(lang),
            &target.map(|p| p.to_string()).unwrap_or("?".to_owned()),
            item.item().map(|x| x.tr_name(lang)).unwrap_or("?"),
        )),
    };

    let reset = {
        let movekind = movekind.clone();
        let players = players.clone();
        let item = item.clone();
        move || { movekind.set(WipMoveKind::None); players.set(Vec::new()); item.reset(); }
    };

    let show_target_dropdown = matches!(*movekind,
        WipMoveKind::Attack | WipMoveKind::OfferTrade | WipMoveKind::UseDiplomat);

    html! {
        <>
            <div class="actionlist">
                {buttons}
            </div>

            // Target player dropdown (Attack / Trade / Diplomat)
            if show_target_dropdown {
                <div class="target-select">
                    <SimpleDropdown<Player>
                        options={other_players.clone()}
                        on_change={Callback::from({ let target = target.clone(); move |p| target.set(Some(p)) })}
                    />
                </div>
            }

            // AnnounceVictory: toggle ally players
            if *movekind == WipMoveKind::AnnounceVictory {
                <div class="ally-select">
                    {for other_players.iter().map(|&p| {
                        let players = players.clone();
                        let selected = players.contains(&p);
                        let active = selected.then_some("is-dark");
                        html! {
                            <button class={classes!("button", active)}
                                onclick={Callback::from(move |_| {
                                    let mut v = (*players).clone();
                                    if selected { v.retain(|&x| x != p); } else { v.push(p); }
                                    players.set(v);
                                })}>
                                {p.to_string()}
                            </button>
                        }
                    })}
                </div>
            }

            // Diplomat: item to demand
            if *movekind == WipMoveKind::UseDiplomat {
                <div class="choose-diplomat">
                    {lang.ask_for()}{" "}
                    <SimpleDropdown<Item>
                        options={DIPLOMAT_ITEM_LIST.to_vec()}
                        on_change={Callback::from(move |x| diplomat_item.set(x))}
                    />
                </div>
            }

            <p class="actiontext">{actiontext}</p>

            if is_victory && *confirming_victory {
                <div class="victory-confirm">
                    <p>{lang.victory_confirm()}</p>
                    <CommandButton text={lang.confirm_yes()} command={upcoming_command} class={"is-success"} onclick={Callback::from({
                        let reset = reset.clone();
                        let confirming_victory = confirming_victory.clone();
                        move |_| { reset(); confirming_victory.set(false); }
                    })} />
                    <button class="button is-light" onclick={Callback::from({
                        let confirming_victory = confirming_victory.clone();
                        move |_| confirming_victory.set(false)
                    })}>{lang.confirm_no()}</button>
                </div>
            } else if is_victory {
                <button class="button actionsubmit is-warning"
                    onclick={Callback::from(move |_| confirming_victory.set(true))}>
                    {lang.submit()}
                </button>
            } else {
                <CommandButton text={lang.submit()} command={upcoming_command} class={"actionsubmit"}
                    onclick={Callback::from(move |_| reset())} />
            }
        </>
    }
}

const DIPLOMAT_ITEM_LIST: &[Item] = &[
    Item::Key, Item::Goblet, Item::BlackPearl, Item::Dagger, Item::Gloves,
    Item::PoisonRing, Item::CastingKnives, Item::Whip, Item::Priviledge,
    Item::Monocle, Item::BrokenMirror, Item::Sextant, Item::Coat,
    Item::Tome, Item::CoatOfArmorOfTheLoge,
];
