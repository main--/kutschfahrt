mod sextant;

use web_protocol::{Player, PerspectiveTradeTriggerState};
use yew::prelude::*;
use super::DoneLookingBtn;

#[derive(Properties, PartialEq)]
pub struct TradeTriggerProps {
    pub is_first_item: bool,
    pub offerer: Player,
    pub target: Player,
    pub trigger: PerspectiveTradeTriggerState,
}

#[function_component(TradeTrigger)]
pub fn trade_trigger(props: &TradeTriggerProps) -> Html {
    let &TradeTriggerProps { is_first_item, offerer, target, ref trigger } = props;

    let (relevant_player, other_player) = if is_first_item { (offerer, target) } else { (target, offerer) };

    match trigger {
        PerspectiveTradeTriggerState::Priviledge { items: None } => html! { <p>{format!("Waiting for {} to look at {}'s items ...", relevant_player, other_player)}</p> },
        PerspectiveTradeTriggerState::Priviledge { items: Some(items) } => html! { <><p>{format!("You see the following items: {:?}", items)}</p><DoneLookingBtn /></> },
        PerspectiveTradeTriggerState::Monocle { faction: None } => html! { <p>{format!("Waiting for {} to look at {}'s faction ...", relevant_player, other_player)}</p> },
        PerspectiveTradeTriggerState::Monocle { faction: Some(faction) } => html! { <><p>{format!("You see that {} is a member of the {:?}.", other_player, faction)}</p><DoneLookingBtn /></> },
        PerspectiveTradeTriggerState::Coat { available_jobs: None } => html! { <p>{format!("Waiting for {} to pick a new job ...", relevant_player)}</p> },
        PerspectiveTradeTriggerState::Coat { available_jobs: Some(jobs) } => html! { <coat::ResolveCoat jobs={jobs.clone()} /> },
        &PerspectiveTradeTriggerState::Sextant { ref item_selections, is_forward } => html! { <sextant::ResolveSextant {is_first_item} {offerer} {target} item_selections={item_selections.clone()} {is_forward}  /> },
    }
}

mod coat;



