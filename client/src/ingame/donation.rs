use web_protocol::Command;
use yew::prelude::*;

use crate::ingame::itemlist::{ItemList, ItemWithIndex};
use crate::ingame::myfaction::MyFaction;
use crate::ingame::myjob::MyJob;
use crate::ingame::playerlist::PlayerList;
use crate::ingame::CommandButton;

#[function_component(ItemDonation)]
pub fn item_donation() -> Html {
    let player = use_state(|| Vec::new());
    let selection = ItemWithIndex::use_new();

    let upcoming_command = match (player.as_slice(), selection.item()) {
        (&[target], Some(item)) => Some(Command::DonateItem { target, item }),
        _ => None,
    };

    html! {
        <>
            <PlayerList selected={Some(player)} />
            <MyFaction />
            <MyJob />
            <ItemList {selection} />

            <CommandButton command={upcoming_command} text={"Submit"} />
        </>
    }
}
