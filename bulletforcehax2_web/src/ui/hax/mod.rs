mod hax_menu;

use std::rc::Rc;

use shared::HaxStateNetwork;
use yew::{html, Component, ContextProvider, Html};

use crate::{hax_ipc::HaxIpc, ui::hax::hax_menu::HaxMenu};

pub struct HaxPage {
    ipc: HaxIpc,
    data: Option<Rc<HaxStateNetwork>>,
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

    fn view(&self, _ctx: &yew::Context<Self>) -> Html {
        html! {
            <>
                if let Some(hax) = &self.data {
                    <ContextProvider<Rc<HaxStateNetwork>> context={hax.clone()}>
                        <HaxMenu />
                    </ContextProvider<Rc<HaxStateNetwork>>>
                }
            </>
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HaxPageMsg::DataReceived(data) => {
                self.data = Some(Rc::new(data));
                true
            }
            HaxPageMsg::SendData(data) => {
                self.ipc.send(data);
                false
            }
        }
    }
}
