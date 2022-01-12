use web_protocol::Item;
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct SelectItemProps {
    pub children: ChildrenWithProps<ItemListEntry>,
    pub on_change: Callback<Option<Item>>,
}

#[function_component(SelectItem)]
pub fn select_item(props: &SelectItemProps) -> Html {
    let sel_index = use_state(|| None);

    html! {
        <div class="itemlist">
            {for props.children.iter().enumerate().map(|(i, child)| {
                let ItemListEntryProps { can_select, item } = *child.props;

                let is_selected = *sel_index == Some(i);
                let selected = if is_selected { Some("selected") } else { None };

                let selectable = if can_select { Some("selectable") } else { None };

                let onchange = props.on_change.clone();
                let sel_index = sel_index.clone();
                html! { <div class={classes!("entry", selected, selectable)} onclick={Callback::from(move |_| {
                    if can_select {
                        if is_selected {
                            sel_index.set(None);
                            onchange.emit(None);
                        } else {
                            sel_index.set(Some(i));
                            onchange.emit(Some(item));
                        }
                    }
                })}>{format!("{:?}", i)}</div> }
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ItemListEntryProps {
    pub can_select: bool,
    pub item: Item,
}

#[function_component(ItemListEntry)]
pub fn item_list_entry(_: &ItemListEntryProps) -> Html {
    unimplemented!("Do not use this directly");
}
