mod ws_worker;

use shared::{FeatureSettings, HaxStateUpdate};

use self::ws_worker::{OpenState, WsComms};

#[derive(Default)]
pub struct HaxIpc {
    comms: Option<WsComms>,
    pub state: Option<(HaxStateUpdate, FeatureSettings)>,
}

impl HaxIpc {
    pub fn try_connect() -> Option<Self> {
        WsComms::connect().map(|comms| Self {
            comms: Some(comms),
            ..Default::default()
        })
    }

    pub fn do_communication_tick(&mut self) {
        let Some(comms) = &mut self.comms else {
            // we don't have communications yet, so there is nothing to do
            return;
        };

        let (received, open_state) = comms.try_recv();
        for message in received {
            match message {
                shared::S2CMessage::InitialState(state, features) => {
                    log::debug!("Received initial state");
                    self.state = Some((state, features));
                }
                shared::S2CMessage::NewGameState(state) => {
                    log::trace!("Received hax state");
                    self.state
                        .as_mut()
                        .expect("state needs to be initialized before receiving hax state")
                        .0 = state;
                }
            }
        }

        if open_state == OpenState::Closed {
            log::debug!("Closing comms");
            self.comms = None;
        }

        // TODO: send out queued messages
    }
}
