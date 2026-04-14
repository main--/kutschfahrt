use std::rc::Rc;

use web_protocol::{Perspective, Player};
use yew::{classes, function_component, html, use_context, Callback, Html, Properties, UseStateHandle};

#[derive(Properties, PartialEq)]
pub struct PlayerListProps {
    #[prop_or_default]
    pub selected: Option<UseStateHandle<Vec<Player>>>,
    #[prop_or_default]
    pub block_select: bool,
}
#[function_component(PlayerList)]
pub fn playerlist(PlayerListProps { selected, block_select }: &PlayerListProps) -> Html {
    let perspective = use_context::<Rc<Perspective>>().unwrap();
    html! {
        <div class="playerlist">
            <div class="entry head">
                <div>{"Player"}</div>
                <div class={"job"}>{"Job"}</div>
                <div>{"Items"}</div>
            </div>
            {for perspective.players.iter().enumerate().map(|(i, p)| {
                let is_you = i == perspective.your_player_index;
                let you = if is_you { Some("you") } else { None };

                let mut class = classes!("entry", you);
                let mut onclick = None;
                if let Some(players) = &selected {
                    if players.contains(&p.player) {
                        class.push("selected")
                    }
                    let can_select = !is_you && !block_select;
                    if can_select {
                        class.push("selectable")
                    }

                    let players = players.clone();
                    let player = p.player;
                    onclick = Some(Callback::from(move |_| {
                        if can_select {
                            let mut p = (*players).clone();
                            if p.contains(&player) {
                                p.retain(|&x| x != player);
                            } else {
                                p.push(player);
                            }
                            players.set(p);
                        }
                    }));
                }

                html! {
                    <div class={class} onclick={onclick}>
                        <div class="name">{p.player.to_string()}</div>
                        <div class="job">{p.job.map(|j| format!("{:?}", j)).unwrap_or("?".to_owned())}</div>
                        <div class="item_count">{p.item_count}</div>
                    </div>
                }
            })}
        </div>
    }
}
