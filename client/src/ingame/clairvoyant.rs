use web_protocol::{Command, Item};
use yew::{function_component, html, use_state, Callback, Html, Properties};

use crate::ingame::CommandButton;

#[derive(Properties, PartialEq)]
pub struct ClairvoyantProps {
    pub item_stack: Vec<Item>,
}

#[function_component(Clairvoyant)]
pub fn clairvoyant(ClairvoyantProps { item_stack}: &ClairvoyantProps) -> Html {
    let stack = use_state(|| item_stack.clone());

    let mut top_items = (*stack).clone();
    top_items.truncate(2);
    let submit = Command::ClairvoyantSetItems { top_items };

    html! {
        <div class="clairvoyant">
            <p>{"Item stack:"}</p>
            <div class="items">
                { for stack.iter().enumerate().map(|(idx, item)| {
                    let stack = stack.clone();
                    html! {
                        <div class="item">
                            <button class="up" onclick={Callback::from(move |_| {
                                let mut s = (*stack).clone();
                                let item = s.remove(idx);
                                s.insert(0, item);
                                stack.set(s);
                            })}>{"↑"}</button>
                            {item}
                        </div>
                    }
                }) }
            </div>
            <CommandButton text={"Submit"} command={submit} class={"submit"} />
        </div>
    }
}
