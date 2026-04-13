use std::rc::Rc;

use web_protocol::{Command, Item, Perspective, PerspectiveTradeTriggerState, Player};
use yew::prelude::*;

use super::{CommandButton, DoneLookingBtn, Lang, Translate, faction_name, itemlist::{ItemList, ItemWithIndex}};

#[derive(Properties, PartialEq)]
pub struct TradeTriggerProps {
    pub myself: Player,
    pub giver: Player,
    pub receiver: Player,
    pub trigger: PerspectiveTradeTriggerState,
}

#[function_component(TradeTrigger)]
pub fn trade_trigger(props: &TradeTriggerProps) -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();
    let &TradeTriggerProps { myself, giver, receiver, ref trigger } = props;
    let am_giver = myself == giver;

    let body = match trigger {
        // ── Privilege: giver sees receiver's items ────────────────────────
        PerspectiveTradeTriggerState::Priviledge { items } => {
            if am_giver {
                match items {
                    Some(items) => html! {
                        <>
                            <p>{ lang.items_of_label(&receiver.to_string()) }</p>
                            <div class="itemlist">
                                { for items.iter().map(|&i| html! {
                                    <div class="entry" data-tooltip={i.tr_tooltip(lang)}>
                                        {i.tr_emoji()}{" "}{i.tr_name(lang)}
                                    </div>
                                })}
                            </div>
                            <DoneLookingBtn />
                        </>
                    },
                    None => html! { <p>{ lang.waiting_others() }</p> },
                }
            } else {
                html! { <p>{ lang.inspecting_items(&giver.to_string(), &receiver.to_string()) }</p> }
            }
        }

        // ── Monocle: giver sees receiver's faction ────────────────────────
        PerspectiveTradeTriggerState::Monocle { faction, three_player_faction_index } => {
            if am_giver {
                match (faction, three_player_faction_index) {
                    // 3-player: first pick which faction card to look at
                    (None, None) => html! {
                        <>
                            <p>{ lang.pick_faction_card(&receiver.to_string()) }</p>
                            <CommandButton text={lang.card(1)} command={Some(Command::ThreePlayerSelectFactionIndex { index: 0 })} />
                            <CommandButton text={lang.card(2)} command={Some(Command::ThreePlayerSelectFactionIndex { index: 1 })} />
                            <CommandButton text={lang.card(3)} command={Some(Command::ThreePlayerSelectFactionIndex { index: 2 })} />
                        </>
                    },
                    // revealed faction (normal or after 3-player pick)
                    (Some(f), idx) => html! {
                        <>
                            <p>{ lang.faction_of(&receiver.to_string(), *idx, faction_name(*f, lang)) }</p>
                            <DoneLookingBtn />
                        </>
                    },
                    // waiting for server response after pick
                    _ => html! { <p>{ lang.waiting_others() }</p> },
                }
            } else {
                html! { <p>{ lang.looking_at_faction(&giver.to_string(), &receiver.to_string()) }</p> }
            }
        }

        // ── Coat: giver picks a new job from available stack ──────────────
        PerspectiveTradeTriggerState::Coat { available_jobs } => {
            if am_giver {
                match available_jobs {
                    Some(jobs) => html! {
                        <>
                            <p>{ lang.pick_new_job() }</p>
                            <div class="actionlist">
                                { for jobs.iter().map(|&job| html! {
                                    <CommandButton
                                        text={job.tr_name(lang)}
                                        command={Some(Command::PickNewJob { job })}
                                    />
                                })}
                            </div>
                        </>
                    },
                    None => html! { <p>{ lang.waiting_others() }</p> },
                }
            } else {
                html! { <p>{ lang.waiting_for_new_job(&giver.to_string()) }</p> }
            }
        }

        // ── Sextant: each player passes an item around ────────────────────
        PerspectiveTradeTriggerState::Sextant { item_selections, is_forward } => {
            html! { <SextantUi myself={myself} giver={giver} item_selections={item_selections.clone()} is_forward={*is_forward} /> }
        }
    };

    html! {
        <div class="trade-trigger">
            { body }
        </div>
    }
}

// ── Sextant sub-component ─────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct SextantUiProps {
    myself: Player,
    giver: Player,
    item_selections: std::collections::HashMap<Player, Item>,
    is_forward: Option<bool>,
}

#[function_component(SextantUi)]
fn sextant_ui(props: &SextantUiProps) -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();
    let perspective = use_context::<Rc<Perspective>>().unwrap();
    let &SextantUiProps { myself, giver, ref item_selections, is_forward } = props;

    let selection = ItemWithIndex::use_new();
    let have_selected = item_selections.contains_key(&myself);
    let all_selected = perspective.players.iter().all(|p| item_selections.contains_key(&p.player));

    html! {
        <>
            <p>{ lang.sextant_intro() }</p>

            // Show existing selections
            if !item_selections.is_empty() {
                <ul>
                    { for item_selections.iter().map(|(p, i)| html! {
                        <li>{ lang.sextant_passes(&p.to_string(), i.tr_name(lang)) }</li>
                    })}
                </ul>
            }

            // My selection (if I haven't chosen yet)
            if !have_selected {
                <>
                    <p>{ lang.sextant_select() }</p>
                    <ItemList selection={selection.clone()} />
                    <CommandButton
                        text={lang.pass_item()}
                        command={selection.item().map(|item| Command::SelectSextantItem { item })}
                    />
                </>
            }

            // Direction selection (giver, after all items chosen)
            if myself == giver && all_selected && is_forward.is_none() {
                <>
                    <p>{ lang.choose_direction() }</p>
                    <CommandButton text={lang.forward()} command={Some(Command::SetSextantDirection { forward: true })} />
                    <CommandButton text={lang.backward()} command={Some(Command::SetSextantDirection { forward: false })} />
                </>
            }

            if have_selected && !(myself == giver && all_selected && is_forward.is_none()) {
                <p>{ lang.waiting_others() }</p>
            }
        </>
    }
}
