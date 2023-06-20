mod ws_worker;

use shared::{C2SMessage, FeatureSettings, HaxStateUpdate};

use crate::ui_context::UiContext;

use self::ws_worker::{OpenState, WsComms};

#[derive(Default)]
pub struct HaxIpc {
    comms: Option<WsComms>,

    // TODO: make this private
    pub state: Option<(HaxStateUpdate, FeatureSettings)>,
}

impl HaxIpc {
    pub fn try_connect() -> Option<Self> {
        WsComms::connect().map(|comms| Self {
            comms: Some(comms),
            ..Default::default()
        })
    }

    pub fn get_ui_context(&self) -> Option<UiContext<'_>> {
        // TODO(perf): I could change FeatureSettings to be a Cow<FeatureSettings>
        if let Some((state, settings)) = &self.state {
            Some(UiContext::new(state, settings))
        } else {
            None
        }
    }

    pub fn recv_comms(&mut self) {
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
    }

    pub fn send_updated_features(&self, new_features: &FeatureSettings) {
        // really frustrating clone, since serialization just needs a &C2S.
        self.comms
            .as_ref()
            .expect("cannot send data without established comms")
            .try_send([C2SMessage::UpdateSettings(new_features.clone())]);
    }
}
