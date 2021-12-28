use web_protocol::{Command, GameCommand};
use yew::prelude::*;

use crate::ingame::Commander;

#[derive(Properties, PartialEq)]
pub struct CommandBtnProps {
    pub command: Option<Command>,
    pub text: &'static str,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(CommandButton)]
pub fn command_button(props: &CommandBtnProps) -> Html {
    let cmd = use_context::<Commander>().unwrap();
    let command = props.command.clone();
    html! { <button class={classes!("button", props.class.clone())} disabled={command.is_none()} onclick={Callback::once(move |_| {
        if let Some(command) = command {
            cmd.cmd(GameCommand::Command(command));
        }
    })}>{props.text}</button> }
}
