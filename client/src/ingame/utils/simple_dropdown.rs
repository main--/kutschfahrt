use std::marker::PhantomData;

use web_sys::HtmlSelectElement;
use yew::prelude::*;


#[derive(Properties, PartialEq)]
pub struct SimpleDropdownProps<T: PartialEq + Copy> {
    pub options: Vec<T>,
    pub on_change: Callback<T>,
}

pub struct SimpleDropdown<T> {
    selected: usize,
    last_selection: Option<T>,
    marker: PhantomData<T>,
}
impl<T: PartialEq + Copy + ToString + 'static> Component for SimpleDropdown<T> {
    type Message = usize;
    type Properties = SimpleDropdownProps<T>;

    fn create(ctx: &Context<Self>) -> Self {
        SimpleDropdown {
            marker: PhantomData,
            selected: 0,
            last_selection: ctx.props().options.iter().copied().next(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let options = &ctx.props().options;
        let selected = self.selected;

        let link = ctx.link().clone();

        html! {
            <div class="select">
                <select onchange={Callback::from(move |e: Event| {
                    let p: usize = e.target_unchecked_into::<HtmlSelectElement>().value().parse().unwrap();
                    link.send_message(p);
                })}>
                    {for options.iter().enumerate().map(|(i, o)| html! { <option value={i.to_string()} selected={selected == i}>{o.to_string()}</option> })}
                </select>
            </div>
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        match self.last_selection {
            Some(x) if !ctx.props().options.contains(&x) => {
                ctx.link().send_message(0usize);
            }
            _ => (),
        }
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.selected = msg;
        let props = ctx.props();
        if let Some(&x) = props.options.get(msg) {
            self.last_selection = Some(x);
            props.on_change.emit(x);
        }
        true
    }
}
