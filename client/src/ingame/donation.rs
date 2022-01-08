use web_protocol::{Perspective, Command};
use yew::prelude::*;

use crate::ingame::CommandButton;

#[derive(Properties, PartialEq)]
pub struct ItemDonationProps {
    pub perspective: Perspective,
}
#[function_component(ItemDonation)]
pub fn item_donation(props: &ItemDonationProps) -> Html {
    let player = use_state(|| None);
    let item = use_state(|| None);

    let upcoming_command = match (*player, *item) {
        (Some(target), Some(item)) => Some(Command::DonateItem { target, item }),
        _ => None,
    };

    html! {
        <>
            <div class="playerlist">
                {for props.perspective.players.iter().enumerate().map(|(i, p)| {
                    let is_you = i == props.perspective.your_player_index;
                    let you = if is_you { Some("you") } else { None };
                    let is_selected = *player == Some(p.player);
                    let selected = if is_selected { Some("selected") } else { None };
                    let can_select = !is_you;
                    let selectable = if can_select { Some("selectable") } else { None };

                    let playerval = p.player;
                    let player = player.clone();

                    html! {
                        <div class={classes!("entry", you, selected, selectable)} onclick={Callback::from(move |_| {
                            if can_select {
                                if is_selected {
                                    player.set(None);
                                } else {
                                    player.set(Some(playerval));
                                }
                            }
                        })}>
                            <div class="name">{p.player.to_string()}</div>
                            <div class="job">{p.job.map(|j| format!("{:?}", j)).unwrap_or("?".to_owned())}</div>
                            <div class="item_count">{p.item_count}</div>
                        </div>
                    }
                })}
            </div>
            <div class="itemlist">
                {for props.perspective.you.items.iter().map(|&i| {
                    let is_selected = *item == Some(i);
                    let selected = if *item == Some(i) { Some("selected") } else { None };
                    let item = item.clone();
                    html! { <div class={classes!("entry", selected, "selectable")} onclick={Callback::from(move |_| {
                        if is_selected {
                            item.set(None);
                        } else {
                            item.set(Some(i));
                        }
                    })}>{format!("{:?}", i)}</div> }
                })}
            </div>

            <CommandButton command={upcoming_command} text={"Submit"} />
        </>
    }
}
