use yew::prelude::*;

#[function_component(PlayPage)]
pub fn app() -> Html {
    log::info!("rendering PlayPage");
    html! {
        <>
            {"go to /game"}
        </>
    }
}
