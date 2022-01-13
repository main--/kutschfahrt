use std::collections::HashMap;
use std::rc::Rc;

use web_protocol::{Perspective, Player, Item, Command};
use yew::prelude::*;

use crate::ingame::CommandButton;

#[derive(Properties, PartialEq)]
pub struct ResolveSextantProps {
    pub is_first_item: bool,
    pub offerer: Player,
    pub target: Player,
    pub item_selections: HashMap<Player, Item>,
    pub is_forward: Option<bool>,
}

#[function_component(ResolveSextant)]
pub fn resolve_sextant(props: &ResolveSextantProps) -> Html {
    let responsible_player = if props.is_first_item { props.offerer } else { props.target };
    let p = use_context::<Rc<Perspective>>().unwrap();
    let me = &p.players[p.your_player_index];
    let ResolveSextantProps { item_selections, is_forward, .. } = props;

    let item = use_state(|| None);

    match is_forward {
        None if responsible_player == me.player => html! {
            <div class="sextant-lr">
                <CommandButton command={Some(Command::SetSextantDirection { forward: true })} text={"Left"} />
                <CommandButton command={Some(Command::SetSextantDirection { forward: false })} text={"Right"} />
            </div>
        },
        None => html! { <p class="sextant-text">{format!("Waiting for {} to determine the direction.", responsible_player)}</p> },
        &Some(is_forward) => {
            let num_players = p.players.len();
            let mut i = p.your_player_index + num_players;
            if is_forward {
                i += 1;
            } else {
                i -= 1;
            }
            i %= num_players;

            let next_player = p.players[i].player;
            match item_selections.get(&me.player) {
                Some(i) => html! { <p class="sextant-text">{format!("You are passing a {:?} to {}.", i, next_player)}</p> },
                None => html! {
                    <>
                        <p class="sextant-text">{format!("Select an item to pass on to {}.", next_player)}</p>
                        <div class="itemlist">
                            {for p.you.items.iter().map(|&i| {
                                let is_selected = *item == Some(i);
                                let selected = if *item == Some(i) { Some("selected") } else { None };

                                let can_select = true;
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
                        <CommandButton command={item.map(|item| Command::SelectSextantItem { item })} text={"Submit"} />
                    </>
                },
            }
        }
    }
}
