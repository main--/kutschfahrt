mod sextant;

use web_protocol::{Perspective, Player, TradeTriggerState, Command};
use yew::prelude::*;

use super::CommandButton;

#[derive(Properties, PartialEq)]
pub struct TradeTriggerProps {
    pub perspective: Perspective,
    pub is_first_item: bool,
    pub offerer: Player,
    pub target: Player,
    pub trigger: TradeTriggerState,
}

#[function_component(TradeTrigger)]
pub fn trade_trigger(props: &TradeTriggerProps) -> Html {
    let &TradeTriggerProps { is_first_item, offerer, target, .. } = props;
    let p = &props.perspective;
    let me = &p.players[p.your_player_index];
    match &props.trigger {
        TradeTriggerState::Priviledge => html! { <DoneLookingBtn /> },
        TradeTriggerState::Monocle => html! { <DoneLookingBtn /> },
        TradeTriggerState::Coat => html! { {"todo"} },
        &TradeTriggerState::Sextant { ref item_selections, is_forward } => html! { <sextant::ResolveSextant perspective={p.clone()} {is_first_item} {offerer} {target} item_selections={item_selections.clone()} {is_forward}  /> },
    }
}

#[function_component(DoneLookingBtn)]
pub fn done_looking_btn() -> Html {
    html! { <CommandButton command={Command::DoneLookingAtThings} text={"Done looking"} /> }
}
