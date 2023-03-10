use yew::{function_component, html, Html};

#[function_component(PlayPage)]
pub fn app() -> Html {
    html! {
        <>
            {"Play page"}
            <br/>
            <a href="/hax" target="_blank">{"Hax"}</a>
        </>
    }
}
