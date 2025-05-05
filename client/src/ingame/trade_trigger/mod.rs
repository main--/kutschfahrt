mod sextant;

use web_protocol::{Command, PerspectiveTradeTriggerState, Player};
use yew::prelude::*;
use crate::ingame::CommandButton;

use super::DoneLookingBtn;

#[derive(Properties, PartialEq)]
pub struct TradeTriggerProps {
    pub myself: Player,
    pub giver: Player,
    pub receiver: Player,
    pub trigger: PerspectiveTradeTriggerState,
}

#[function_component(TradeTrigger)]
pub fn trade_trigger(props: &TradeTriggerProps) -> Html {
    let &TradeTriggerProps { myself, giver, receiver, ref trigger } = props;

    match trigger {
        PerspectiveTradeTriggerState::Priviledge { items: None } => html! { <p>{format!("Waiting for {} to look at {}'s items ...", giver, receiver)}</p> },
        PerspectiveTradeTriggerState::Priviledge { items: Some(items) } => html! { <><p>{format!("You see the following items: {:?}", items)}</p><DoneLookingBtn /></> },
        PerspectiveTradeTriggerState::Monocle { faction: None } if giver == myself => html! {
            html! {
                <>
                    {"Pick a faction card to look at:"}
                    <CommandButton text={"1"} command={Some(Command::ThreePlayerSelectFactionIndex { index: 0 })} />
                    <CommandButton text={"2"} command={Some(Command::ThreePlayerSelectFactionIndex { index: 1 })} />
                    <CommandButton text={"3"} command={Some(Command::ThreePlayerSelectFactionIndex { index: 2 })} />
                </>
            }
        },
        PerspectiveTradeTriggerState::Monocle { faction: None } => html! { <p>{format!("Waiting for {} to look at {}'s faction ...", giver, receiver)}</p> },
        PerspectiveTradeTriggerState::Monocle { faction: Some(faction) } => html! { <><p>{format!("You see that {} is a member of the {:?}.", receiver, faction)}</p><DoneLookingBtn /></> },
        PerspectiveTradeTriggerState::Coat { available_jobs: None } => html! { <p>{format!("Waiting for {} to pick a new job ...", giver)}</p> },
        PerspectiveTradeTriggerState::Coat { available_jobs: Some(jobs) } => html! { <coat::ResolveCoat jobs={jobs.clone()} /> },
        &PerspectiveTradeTriggerState::Sextant { ref item_selections, is_forward } => html! { <sextant::ResolveSextant responsible_player={giver} item_selections={item_selections.clone()} {is_forward}  /> },
    }
}

mod coat;

// TODO:
// hellseher
// diplomat

// es braucht dringend dudum

// foliant should be indicated to the others

// turn number

// wappen der loge sieg verkünden


// funny stats like "zacharias hat 3x den job gewechselt" "zacharias hat 90% der kämpfe verloren"