use web_protocol::Command;
use yew::prelude::*;
use super::CommandButton;

#[function_component(DoneLookingBtn)]
pub fn done_looking_btn() -> Html {
    html! { <CommandButton command={Command::DoneLookingAtThings} text={"Done looking"} /> }
}
