use yew::prelude::*;
use web_protocol::{Player, Item, PlayerState, Command};

use crate::ingame::itemlist::{ItemList, ItemWithIndex};
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
    let selection = ItemWithIndex::use_new();
    let reject = match props.item {
        Item::BlackPearl | Item::BrokenMirror => None,
        _ => Some(Command::RejectTrade),
    };

    let forbidden_combo = vec![Item::BagGoblet, Item::BagKey];
    let blocklist = if forbidden_combo.contains(&props.item) {
        forbidden_combo
    } else {
        Vec::new()
    };

    let item = selection.item();

    html! {
        <div class="item-offer">
            <ItemList {selection} {blocklist} />

            <div class="text">{format!("{} is offering you a {}", props.offerer, props.item)}</div>
            <CommandButton class="is-green" text={"Accept"} command={item.map(|item| Command::AcceptTrade { item })} />
            <CommandButton class="is-red" text={"Decline"} command={reject} />
        </div>
    }
}