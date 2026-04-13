use std::rc::Rc;

use web_protocol::{Command, GameCommand, Item};
use yew::prelude::*;

use super::{Commander, Lang, Translate};

// ─── SelectItem context ──────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub struct SelectItemCtx {
    pub on_change: Callback<Option<Item>>,
    pub selected: Option<Item>,
}

#[derive(Properties, PartialEq)]
pub struct SelectItemProps {
    pub on_change: Callback<Option<Item>>,
    #[prop_or_default]
    pub children: Children,
}

/// Flex container that provides selection context to `ItemListEntry` children.
/// Calls `on_change` with the newly selected item (or None on deselect).
#[function_component(SelectItem)]
pub fn select_item(props: &SelectItemProps) -> Html {
    let selected = use_state(|| None::<Item>);
    let on_change_prop = props.on_change.clone();
    let selected_handle = selected.clone();

    let ctx = SelectItemCtx {
        on_change: Callback::from(move |item: Option<Item>| {
            selected_handle.set(item);
            on_change_prop.emit(item);
        }),
        selected: *selected,
    };

    html! {
        <ContextProvider<SelectItemCtx> context={ctx}>
            <div class="itemselect">
                { for props.children.iter() }
            </div>
        </ContextProvider<SelectItemCtx>>
    }
}

// ─── ItemListEntry ───────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct ItemListEntryProps {
    pub item: Item,
    pub can_select: bool,
}

#[function_component(ItemListEntry)]
pub fn item_list_entry(props: &ItemListEntryProps) -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();
    let ctx = use_context::<SelectItemCtx>();
    let item = props.item;
    let can_select = props.can_select;

    let is_selected = ctx.as_ref().map_or(false, |c| c.selected == Some(item));

    let onclick = ctx.map(|c| {
        Callback::from(move |_: MouseEvent| {
            if can_select {
                if c.selected == Some(item) {
                    c.on_change.emit(None);
                } else {
                    c.on_change.emit(Some(item));
                }
            }
        })
    });

    html! {
        <div
            class={classes!(
                "item-entry",
                can_select.then_some("selectable"),
                is_selected.then_some("selected")
            )}
            data-tooltip={item.tr_desc(lang)}
            onclick={onclick}
        >
            { item.tr_name(lang) }
        </div>
    }
}

// ─── CommandButton ────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct CommandButtonProps {
    pub text: AttrValue,
    pub command: Option<Command>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub onclick: Callback<()>,
}

#[function_component(CommandButton)]
pub fn command_button(props: &CommandButtonProps) -> Html {
    let cmd_ctx = use_context::<Commander>().unwrap();
    let command = props.command.clone();
    let user_onclick = props.onclick.clone();
    // pending: true immediately after click, reset when command prop changes
    let pending = use_state(|| false);

    // Reset pending whenever the command value changes (new render with new state)
    {
        let pending = pending.clone();
        use_effect_with(props.command.clone(), move |_| {
            pending.set(false);
        });
    }

    let disabled = command.is_none() || *pending;

    html! {
        <button
            class={classes!("button", props.class.clone())}
            disabled={disabled}
            onclick={Callback::from(move |_: MouseEvent| {
                if let Some(cmd) = command.clone() {
                    pending.set(true);
                    cmd_ctx.cmd(GameCommand::Command(cmd));
                    user_onclick.emit(());
                }
            })}
        >
            { &props.text }
        </button>
    }
}

// ─── DoneLookingBtn ───────────────────────────────────────────────────────────

#[function_component(DoneLookingBtn)]
pub fn done_looking_btn() -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();
    html! {
        <CommandButton text={lang.done()} command={Some(Command::DoneLookingAtThings)} />
    }
}

// ─── SimpleDropdown ───────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct SimpleDropdownProps<T: PartialEq + Clone + std::fmt::Display + 'static> {
    pub options: Vec<T>,
    pub on_change: Callback<T>,
}

#[function_component(SimpleDropdown)]
pub fn simple_dropdown<T: PartialEq + Clone + std::fmt::Display + 'static>(
    props: &SimpleDropdownProps<T>,
) -> Html {
    let options = props.options.clone();
    let on_change = props.on_change.clone();
    let options_for_cb = Rc::new(options.clone());

    html! {
        <div class="select">
            <select onchange={Callback::from(move |e: Event| {
                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                if let Ok(idx) = select.value().parse::<usize>() {
                    if let Some(opt) = options_for_cb.get(idx) {
                        on_change.emit(opt.clone());
                    }
                }
            })}>
                { for options.iter().enumerate().map(|(i, opt)| html! {
                    <option value={i.to_string()}>{ opt.to_string() }</option>
                })}
            </select>
        </div>
    }
}
