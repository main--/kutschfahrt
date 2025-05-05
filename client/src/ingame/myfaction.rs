use std::rc::Rc;

use web_protocol::{FactionKind, Perspective};
use yew::{function_component, html, use_context, Html};

#[function_component(MyFaction)]
pub fn my_faction() -> Html {
    let perspective = use_context::<Rc<Perspective>>().unwrap();

    let body = match perspective.you.faction {
        FactionKind::Normal(faction) => format!("Your faction: {faction:?}"),
        FactionKind::ThreePlayer(factions) => format!("Your faction cards: {:?}, {:?}, {:?}", factions[0], factions[1], factions[2]),
    };

    html! {
        <div class="yourfaction">
            {body}
        </div>
    }
}
