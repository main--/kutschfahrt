use web_protocol::{Command, Player};
use yew::prelude::*;

use crate::ingame::itemlist::ItemWithIndex;
use crate::ingame::{CommandButton, Lang};

#[derive(Properties, PartialEq)]
pub struct ItemDonationProps {
    pub players: UseStateHandle<Vec<Player>>,
    pub item: ItemWithIndex,
}

#[function_component(ItemDonation)]
pub fn item_donation(props: &ItemDonationProps) -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();
    let ItemDonationProps { players, item } = props;

    let upcoming_command = match (players.as_slice(), item.item()) {
        (&[target], Some(item)) => Some(Command::DonateItem { target, item }),
        _ => None,
    };

    html! {
        <>
            <p>{lang.donate_prompt()}</p>
            <CommandButton command={upcoming_command} text={lang.submit()} />
        </>
    }
}
