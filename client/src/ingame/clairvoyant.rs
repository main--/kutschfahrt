use web_protocol::{Command, Item};
use yew::{function_component, html, use_context, use_state, Callback, Html, Properties};

use crate::ingame::{CommandButton, Lang, Translate};

#[derive(Properties, PartialEq)]
pub struct ClairvoyantProps {
    pub item_stack: Vec<Item>,
}

#[function_component(Clairvoyant)]
pub fn clairvoyant(ClairvoyantProps { item_stack}: &ClairvoyantProps) -> Html {
    let stack = use_state(|| item_stack.clone());
    let lang = use_context::<Lang>().unwrap_or_default();

    let mut top_items = (*stack).clone();
    top_items.truncate(2);
    let submit = Command::ClairvoyantSetItems { top_items };

    html! {
        <div class="clairvoyant">
            <p>{lang.item_stack_label()}</p>
            <div class="items">
                { for stack.iter().enumerate().map(|(idx, &item)| {
                    let stack = stack.clone();
                    html! {
                        <div class="item">
                            <button class="up" onclick={Callback::from(move |_| {
                                let mut s = (*stack).clone();
                                let it = s.remove(idx);
                                s.insert(0, it);
                                stack.set(s);
                            })}>{"↑"}</button>
                            {item.tr_name(lang)}
                        </div>
                    }
                }) }
            </div>
            <CommandButton text={lang.submit()} command={submit} class={"submit"} />
        </div>
    }
}
