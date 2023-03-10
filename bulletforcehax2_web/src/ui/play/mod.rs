use yew::{function_component, html, Html};

const SCRIPT: &str = include_str!("script.js");

#[function_component(PlayPage)]
pub fn app() -> Html {
    log::info!("rendering PlayPage");
    html! {
        <>
            <script id="loader-js" src="/game_assets/loader.js"></script>
            <canvas id="unity-canvas" width="960" height="600"></canvas>
            <script>{SCRIPT}</script>
            <ul>
                <li><a href="/hax" target="_blank">{"Hax"}</a></li>
            </ul>
        </>
    }
}
