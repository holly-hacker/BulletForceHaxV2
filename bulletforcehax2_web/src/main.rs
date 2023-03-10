use log::info;
use ui::{HaxPage, NotFoundPage, PlayPage};
use yew::prelude::*;
use yew_router::{BrowserRouter, Routable, Switch};

mod ui;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    info!("Hello, world!");
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Play => html! {<PlayPage/>},
        Route::Hax => html! {<HaxPage/>},
        Route::NotFound => html! {<NotFoundPage/>},
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Play,
    #[at("/hax")]
    Hax,
    #[not_found]
    #[at("/404")]
    NotFound,
}
