use yew::{html, Component, Html};

use crate::hax_ipc::HaxIpc;

pub struct HaxPage {
    ipc: HaxIpc,
    data: String,
}

pub enum HaxPageMsg {
    DataReceived(String),
    SendData(String),
}

impl Component for HaxPage {
    type Message = HaxPageMsg;
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        let on_message = ctx.link().callback(HaxPageMsg::DataReceived);
        Self {
            ipc: HaxIpc::new_connect(on_message),
            data: "initial data".into(),
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let onclick = ctx
            .link()
            .callback(|x| HaxPageMsg::SendData(format!("clicked button: {x:?}")));
        html! {
            <>
                {format!("data: {}", self.data)}
                <br/>
                <button {onclick}>{"Click me"}</button>
            </>
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HaxPageMsg::DataReceived(data) => {
                self.data = data;
                true
            }
            HaxPageMsg::SendData(data) => {
                self.ipc.send(data);
                false
            }
        }
    }
}
