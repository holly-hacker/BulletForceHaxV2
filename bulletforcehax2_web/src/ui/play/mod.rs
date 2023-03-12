use yew::prelude::*;

const SCRIPT: &str = include_str!("script.js");

#[function_component(PlayPage)]
pub fn app() -> Html {
    log::info!("rendering PlayPage");
    html! {
        <>
            <ybc::Hero body={html! {<ybc::Title>{"BulletForceHaxV2"}</ybc::Title>}} classes={classes!("is-primary")} />
            <ybc::Container>
                <script id="loader-js" src="/game_assets/loader.js"></script>
                <canvas id="unity-canvas" width="960" height="600"></canvas>
                <script>{SCRIPT}</script>
                <div class="is-clearfix" />
                <ybc::ButtonAnchor href="/hax" target="_blank">{"Open Hax"}</ybc::ButtonAnchor>
            </ybc::Container>
        </>
    }
}
