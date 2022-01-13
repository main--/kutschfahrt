use web_protocol::{Job, Command};
use yew::prelude::*;
use crate::ingame::{SimpleDropdown, CommandButton};

#[derive(Properties, PartialEq)]
pub struct ResolveCoatProps {
    pub jobs: Vec<Job>,
}
#[function_component(ResolveCoat)]
pub fn resolve_coat(props: &ResolveCoatProps) -> Html {
    let j = use_state(|| props.jobs.iter().next().copied().unwrap());
    html! {
        <>
            <p>{"Pick a new job:"}</p>
            <SimpleDropdown<Job> options={props.jobs.clone()} on_change={Callback::from({ let j = j.clone(); Callback::from(move |x| j.set(x)) })} />
            <CommandButton text={"Pick"} command={Some(Command::PickNewJob { job: *j })} />
        </>
    }
}
