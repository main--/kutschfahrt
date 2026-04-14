use std::rc::Rc;

use web_protocol::{Item, Perspective};
use yew::{classes, function_component, hook, html, use_context, use_state, Callback, Html, Properties, UseStateHandle};

#[derive(PartialEq, Clone)]
pub struct ItemWithIndex(UseStateHandle<Option<(Item, usize)>>);
impl ItemWithIndex {
    #[hook]
    pub fn use_new() -> ItemWithIndex {
        ItemWithIndex(use_state(|| None))
    }
    pub fn reset(&self) {
        self.0.set(None);
    }
    pub fn set(&self, item: Item, index: usize) {
        self.0.set(Some((item, index)));
    }
    pub fn item(&self) -> Option<Item> {
        self.0.map(|x| x.0)
    }
    pub fn index(&self) -> Option<usize> {
        self.0.map(|x| x.1)
    }
}

#[derive(Properties, PartialEq)]
pub struct ItemListProps {
    #[prop_or_default]
    pub selection: Option<ItemWithIndex>,
    #[prop_or_default]
    pub blocklist: Vec<Item>,
}
#[function_component(ItemList)]
pub fn playerlist(ItemListProps { selection, blocklist }: &ItemListProps) -> Html {
    let perspective = use_context::<Rc<Perspective>>().unwrap();
    html! {
        <>
            {"Your items:"}
            <div class="itemlist">
                {for perspective.you.items.iter().enumerate().map(|(idx, &i)| {
                    let is_selected = selection.as_ref().map_or(false, |x| x.index() == Some(idx));
                    let selected = if is_selected { Some("selected") } else { None };
                    let can_select = selected.is_some() && !blocklist.contains(&i);
                    let selectable = if can_select { Some("selectable") } else { None };
                    let selection = selection.clone();
                    html! { <div class={classes!("entry", selected, selectable)} onclick={Callback::from(move |_| {
                        if let Some(selection) = &selection {
                            if is_selected {
                                selection.reset();
                            } else {
                                selection.set(i, idx);
                            }
                        }
                    })}>{i.to_string()}</div> }
                })}
            </div>
        </>
    }
}
