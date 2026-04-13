use std::rc::Rc;

use web_protocol::{Faction, FactionKind, Perspective};
use yew::{function_component, html, use_context, Html};

use super::{Lang, faction_name, faction_victory_tip};

#[function_component(MyFaction)]
pub fn my_faction() -> Html {
    let perspective = use_context::<Rc<Perspective>>().unwrap();
    let lang = use_context::<Lang>().unwrap_or_default();

    let body = match perspective.you.faction {
        FactionKind::Normal(faction) => html! {
            <div class="yourfaction">
                {lang.your_faction()}{" "}
                <span data-tooltip={faction_victory_tip(faction, lang)}>
                    {faction_name(faction, lang)}
                </span>
            </div>
        },
        FactionKind::ThreePlayer(factions) => {
            let names = factions.map(|f| faction_name(f, lang));
            html! {
                <div class="yourfaction">
                    {lang.your_faction_cards()}{" "}
                    <span data-tooltip={faction_victory_tip(majority_faction(factions), lang)}>
                        {format!("{}, {}, {}", names[0], names[1], names[2])}
                    </span>
                </div>
            }
        }
    };

    body
}

/// Returns the faction that appears on the majority of the three cards.
fn majority_faction(factions: [Faction; 3]) -> Faction {
    let orders = factions.iter().filter(|&&f| f == Faction::Order).count();
    if orders >= 2 { Faction::Order } else { Faction::Brotherhood }
}
