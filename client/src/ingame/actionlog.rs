use std::rc::Rc;

use web_protocol::{ActionLogEntry, Perspective};
use yew::{function_component, html, use_context, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct ActionLogProps {
}
#[function_component(ActionLog)]
pub fn actionlog(ActionLogProps {}: &ActionLogProps) -> Html {
    let perspective = use_context::<Rc<Perspective>>().unwrap();

    html! {
        <div class="actionlog">
            { for perspective.action_log.iter().cloned().map(|action| html! { <Entry {action} /> }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ActionLogEntryProps {
    action: ActionLogEntry,
}
#[function_component(Entry)]
fn entry(ActionLogEntryProps { action }: &ActionLogEntryProps) -> Html {
    let body = match action {
        &ActionLogEntry::Pass { actor } => format!("{actor} passed."),
        &ActionLogEntry::AnnounceVictory { actor } => format!("{actor} announced victory."),
        &ActionLogEntry::UseDiplomat { actor, target, item, success: true } => format!("{actor} asked {target} for a {item}. They exchanged items."),
        &ActionLogEntry::UseDiplomat { actor, target, item, success: false } => format!("{actor} asked {target} for a {item}, but {target} did not have one."),
        &ActionLogEntry::UseClairvoyant { actor } => format!("{actor} reordered the item stack."),
        &ActionLogEntry::TradeOffer { offerer, target, accepted } => format!("{offerer} offered a trade to {target}. The trade was {}.", if accepted { "accepted" } else { "declined" }),
        &ActionLogEntry::Attack { attacker, target } => format!("{attacker} attacked {target}."),
        &ActionLogEntry::TradeTrigger { giver, receiver, item } => format!("{giver} passed a {item} to {receiver}."),
        &ActionLogEntry::DonateItem { giver, receiver } => format!("{giver} donates an item to {receiver}."),
    };
    html! {
        <div class="entry">
            {body}
        </div>
    }
}
