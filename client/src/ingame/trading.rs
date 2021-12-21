use yew::prelude::*;
use web_protocol::{Player, Item, PlayerState, GameCommand, Command};

use crate::ingame::Commander;

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
    let cmd = use_context::<Commander>().unwrap();
    let cmd2 = cmd.clone();
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
            <button class="button is-green" disabled={item.is_none()} onclick={Callback::from(move |_| {
                if let Some(item) = *item {
                    cmd.cmd(GameCommand::Command(Command::AcceptTrade { item }));
                }
            })}>{"Accept"}</button>
            <button class="button is-red" onclick={Callback::from(move |_| {
                cmd2.cmd(GameCommand::Command(Command::RejectTrade));
            })}>{"Decline"}</button>
        </div>
    }
}