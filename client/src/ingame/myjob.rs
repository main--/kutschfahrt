use std::rc::Rc;

use web_protocol::Perspective;
use yew::{function_component, html, use_context, Html, Properties};

use super::{Lang, Translate};

#[derive(Properties, PartialEq)]
pub struct MyJobProps {
}
#[function_component(MyJob)]
pub fn my_job(MyJobProps {}: &MyJobProps) -> Html {
    let perspective = use_context::<Rc<Perspective>>().unwrap();
    let lang = use_context::<Lang>().unwrap_or_default();
    let job = perspective.you.job;
    let revealed = if perspective.you.job_is_visible { lang.revealed() } else { lang.not_revealed() };

    html! {
        <div class="yourjob">
            {lang.your_job()}
            {" "}
            <span data-tooltip={job.tr_tooltip(lang)}>{job.tr_name(lang)}</span>
            {format!(" ({})", revealed)}
        </div>
    }
}
