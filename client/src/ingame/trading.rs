use yew::prelude::*;
use web_protocol::{Player, Item, PlayerState, Command};

use crate::ingame::CommandButton;

#[derive(Properties, PartialEq)]
pub struct TradeOfferProps {
    pub you: PlayerState,
    pub offerer: Player,
    pub item: Item,
    pub stack_empty: bool,
}
#[function_component(TradeOffer)]
pub fn trade_offer(props: &TradeOfferProps) -> Html {
    let item = use_state(|| None);
    html! {
        <div class="item-offer">
            <div class="itemlist">
                {for props.you.items.iter().map(|&i| {
                    let is_selected = *item == Some(i);
                    let selected = if *item == Some(i) { Some("selected") } else { None };
                    // TODO: letztes hemd (lol). anything else?
                    let forbidden_combo = [Item::BagGoblet, Item::BagKey];
                    let can_select = props.stack_empty || !(forbidden_combo.contains(&i) && forbidden_combo.contains(&props.item));
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

            <div class="text">{format!("{} is offering you a {:?}", props.offerer, props.item)}</div>
            <CommandButton class="is-green" text={"Accept"} command={item.map(|item| Command::AcceptTrade { item })} />
            <CommandButton class="is-red" text={"Decline"} command={Some(Command::RejectTrade)} />
        </div>
    }
}