use yew::prelude::*;
use web_protocol::{Player, Item, PlayerState, Command};

use crate::ingame::itemlist::ItemWithIndex;
use crate::ingame::{CommandButton, Lang, Translate};

#[derive(Properties, PartialEq)]
pub struct TradeOfferProps {
    pub you: PlayerState,
    pub offerer: Player,
    pub item: Item,
    pub stack_empty: bool,
    pub item_selection: ItemWithIndex,
}

#[function_component(TradeOffer)]
pub fn trade_offer(props: &TradeOfferProps) -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();
    let must_accept_reason: Option<&'static str> = match props.item {
        Item::BlackPearl  => Some(lang.black_pearl_must_accept()),
        Item::BrokenMirror => Some(lang.broken_mirror_must_accept()),
        _ => None,
    };
    let reject = must_accept_reason.is_none().then_some(Command::RejectTrade);

    let item = props.item_selection.item();

    html! {
        <div class="item-offer">
            <div class="text">
                {lang.offering_before(&props.offerer.to_string())}
                <span data-tooltip={props.item.tr_tooltip(lang)}><strong>{props.item.tr_emoji()}{" "}{props.item.tr_name(lang)}</strong></span>
                {lang.offering_after()}
            </div>
            <p>{lang.select_item_hint()}</p>
            <CommandButton class="is-success" text={lang.accept()} command={item.map(|item| Command::AcceptTrade { item })} />
            if let Some(reason) = must_accept_reason {
                <span class="disabled-btn-wrap" data-tooltip={reason}>
                    <button class="button is-danger" disabled={true}>{lang.decline()}</button>
                </span>
            } else {
                <CommandButton class="is-danger" text={lang.decline()} command={reject} />
            }
        </div>
    }
}
