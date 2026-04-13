use std::rc::Rc;

use web_protocol::{FactionKind, Perspective};
use yew::{function_component, html, use_context, Html};

use super::{Lang, faction_name};

#[function_component(MyFaction)]
pub fn my_faction() -> Html {
    let perspective = use_context::<Rc<Perspective>>().unwrap();
    let lang = use_context::<Lang>().unwrap_or_default();

    let body = match perspective.you.faction {
        FactionKind::Normal(faction) => format!("{} {}", lang.your_faction(), faction_name(faction, lang)),
        FactionKind::ThreePlayer(factions) => format!("{} {}, {}, {}",
            lang.your_faction_cards(),
            faction_name(factions[0], lang),
            faction_name(factions[1], lang),
            faction_name(factions[2], lang)),
    };

    html! {
        <div class="yourfaction">
            {body}
        </div>
    }
}
