use std::sync::Arc;

use futures_util::lock::Mutex;
use photon_lib::{
    photon_data_type::PhotonDataType,
    photon_message::PhotonMessage,
    realtime::{
        constants::{event_code, parameter_code},
        EventDataBased, RoomInfo,
    },
};
use tracing::{info, trace};

use crate::hax::HaxState;

impl HaxState {
    #[allow(clippy::ptr_arg)]
    pub fn webrequest_hook_onrequest(
        _hax: Arc<Mutex<Self>>,
        _url: &hyper::Uri,
        _bytes: &mut Vec<u8>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    #[allow(clippy::ptr_arg)]
    pub fn webrequest_hook_onresponse(
        _hax: Arc<Mutex<Self>>,
        _url: &hyper::Uri,
        _bytes: &mut Vec<u8>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    #[allow(clippy::ptr_arg)]
    pub fn websocket_hook(
        hax: Arc<Mutex<Self>>,
        data: &mut Vec<u8>,
        direction: &'static str,
    ) -> anyhow::Result<()> {
        let mut photon_message = PhotonMessage::from_websocket_bytes(&mut data.as_slice())?;
        trace!("{direction} data: {photon_message:?}");

        // TODO: check if on lobby socket
        if let PhotonMessage::EventData(event) = &mut photon_message {
            if event.code == event_code::GAME_LIST || event.code == event_code::GAME_LIST_UPDATE {
                let spoofed_max_players =
                    futures::executor::block_on(hax.lock()).lobby_spoofed_max_players;
                let game_list = event.parameters.get_mut(&parameter_code::GAME_LIST);
                if let Some(PhotonDataType::Hashtable(games)) = game_list {
                    for (k, v) in games {
                        if let (
                            PhotonDataType::String(game_name),
                            PhotonDataType::Hashtable(props),
                        ) = (k, v)
                        {
                            // tracing::debug!("Before: {props:?}");
                            let mut room_info = RoomInfo::from_hashtable(props);
                            info!("room {game_name}: {room_info:?}");
                            if room_info.removed != Some(true) {
                                if let Some(new_max_players) = spoofed_max_players {
                                    // NOTE: for some reason, max_players gets incremented by 1 ingame
                                    room_info.max_players = Some(new_max_players);
                                }
                            }
                            room_info.into_hashtable(props);
                            // tracing::debug!("After: {props:?}");
                        }
                    }
                }
            }
        }

        let mut buf: Vec<u8> = vec![];
        photon_message.to_websocket_bytes(&mut buf)?;
        *data = buf;

        Ok(())
    }
}
