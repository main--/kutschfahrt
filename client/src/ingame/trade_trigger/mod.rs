mod sextant;

use web_protocol::{Perspective, Player, TradeTriggerState};
use yew::prelude::*;

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
        &TradeTriggerState::Sextant { ref item_selections, is_forward } => html! { <sextant::ResolveSextant perspective={p.clone()} {is_first_item} {offerer} {target} item_selections={item_selections.clone()} {is_forward}  /> },
        _ => todo!(),
    }
}
