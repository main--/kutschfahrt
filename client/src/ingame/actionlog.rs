use std::rc::Rc;

use web_protocol::Perspective;
use yew::{function_component, html, use_context, Html, Properties};

use super::{Lang, action_log_text};

#[derive(Properties, PartialEq)]
pub struct ActionLogProps {
}
#[function_component(ActionLog)]
pub fn actionlog(ActionLogProps {}: &ActionLogProps) -> Html {
    let perspective = use_context::<Rc<Perspective>>().unwrap();
    let lang = use_context::<Lang>().unwrap_or_default();

    html! {
        <div class="actionlog">
            { for perspective.action_log.iter().rev().map(|action| {
                let body = action_log_text(action, lang);
                html! { <div class="entry">{body}</div> }
            }) }
            <div class="entry game-start">{lang.game_start()}</div>
        </div>
    }
}
