use std::rc::Rc;

use web_protocol::Perspective;
use yew::{function_component, html, use_context, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct MyJobProps {
}
#[function_component(MyJob)]
pub fn my_job(MyJobProps {}: &MyJobProps) -> Html {
    let perspective = use_context::<Rc<Perspective>>().unwrap();

    html! {
        <div class="yourjob">
            {"Your job: "}{perspective.you.job}{format!(" ({}revealed)", if perspective.you.job_is_visible { "" } else { "not " })}
        </div>
    }
}
