mod hax_menu;

use std::rc::Rc;

use shared::{HaxStateUpdate, S2CMessage};
use yew::{html, Component, ContextProvider, Html};

use crate::{hax_ipc::HaxIpc, ui::hax::hax_menu::HaxMenu};

pub struct HaxPage {
    ipc: HaxIpc,
    game_state: Option<Rc<HaxStateUpdate>>,
}

pub enum HaxPageMsg {
    MessageReceived(S2CMessage),
    #[allow(unused)]
    SendData(String),
}

impl Component for HaxPage {
    type Message = HaxPageMsg;
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        // bind incoming messages to a component message
        let message_received_callback = ctx.link().callback(HaxPageMsg::MessageReceived);
        Self {
            ipc: HaxIpc::connect(message_received_callback),
            game_state: None,
        }
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> Html {
        html! {
            <>
                if let Some(hax) = &self.game_state {
                    <ContextProvider<Rc<HaxStateUpdate>> context={hax.clone()}>
                        <HaxMenu />
                    </ContextProvider<Rc<HaxStateUpdate>>>
                }
            </>
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HaxPageMsg::MessageReceived(message) => {
                match message {
                    S2CMessage::InitialState(game, _) | S2CMessage::NewGameState(game) => {
                        self.game_state = Some(Rc::new(game))
                    }
                }
                true
            }
            HaxPageMsg::SendData(data) => {
                self.ipc.send(data);
                false
            }
        }
    }
}
