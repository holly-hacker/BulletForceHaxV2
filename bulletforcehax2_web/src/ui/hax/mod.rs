use shared::HaxStateNetwork;
use yew::{html, Component, Html};

use crate::hax_ipc::HaxIpc;

pub struct HaxPage {
    ipc: HaxIpc,
    data: Option<HaxStateNetwork>,
}

pub enum HaxPageMsg {
    DataReceived(HaxStateNetwork),
    SendData(String),
}

impl Component for HaxPage {
    type Message = HaxPageMsg;
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        let on_message = ctx.link().callback(HaxPageMsg::DataReceived);
        Self {
            ipc: HaxIpc::new_connect(on_message),
            data: None,
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let onclick = ctx
            .link()
            .callback(|x| HaxPageMsg::SendData(format!("clicked button: {x:?}")));
        html! {
            <>
                <pre>{format!("data: {:#?}", self.data)}</pre>
                <button {onclick}>{"Click me"}</button>
            </>
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HaxPageMsg::DataReceived(data) => {
                self.data = Some(data);
                true
            }
            HaxPageMsg::SendData(data) => {
                self.ipc.send(data);
                false
            }
        }
    }
}
