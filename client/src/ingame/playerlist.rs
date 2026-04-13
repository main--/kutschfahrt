use std::rc::Rc;

use web_protocol::{Perspective, Player};
use yew::{classes, function_component, html, use_context, Callback, Html, Properties, UseStateHandle};

use super::{Lang, Translate};

#[derive(Properties, PartialEq)]
pub struct PlayerListProps {
    #[prop_or_default]
    pub selected: Option<UseStateHandle<Vec<Player>>>,
    #[prop_or_default]
    pub block_select: bool,
    #[prop_or_default]
    pub active_player: Option<Player>,
}
#[function_component(PlayerList)]
pub fn playerlist(PlayerListProps { selected, block_select, active_player }: &PlayerListProps) -> Html {
    let perspective = use_context::<Rc<Perspective>>().unwrap();
    let lang = use_context::<Lang>().unwrap_or_default();
    html! {
        <div class="playerlist">
            <div class="entry head">
                <div>{lang.player_col()}</div>
                <div class={"job"}>{lang.job_col()}</div>
                <div>{lang.items_col()}</div>
            </div>
            {for perspective.players.iter().enumerate().map(|(i, p)| {
                let is_you = i == perspective.your_player_index;
                let is_active = *active_player == Some(p.player);
                let you = if is_you { Some("you") } else { None };
                let active = if is_active { Some("active-turn") } else { None };

                let mut class = classes!("entry", you, active);
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
                        <div class="job" data-tooltip={p.job.map(|j| j.tr_tooltip(lang)).unwrap_or_default()}>
                            {p.job.map(|j| j.tr_name(lang).to_string()).unwrap_or_else(|| "?".to_owned())}
                        </div>
                        <div class="item_count">{p.item_count}</div>
                    </div>
                }
            })}
        </div>
    }
}
