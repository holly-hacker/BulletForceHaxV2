use log::info;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div>{"Hello from yew!"}</div>
    }
}
fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    info!("Hello, world!");
    yew::Renderer::<App>::new().render();
}
