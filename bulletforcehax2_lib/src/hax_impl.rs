use std::sync::Arc;

use futures_util::lock::Mutex;
use photon_lib::{
    photon_data_type::PhotonDataType,
    photon_message::PhotonMessage,
    realtime::{
        constants::{event_code, parameter_code},
        PhotonMapConversion, Player, RoomInfo,
    },
};
use tracing::{debug, info, trace};

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

        let x = match &photon_message {
            PhotonMessage::OperationRequest(r) => Some(("OperationRequest", r.operation_code)),
            PhotonMessage::OperationResponse(r) => Some(("OperationResponse", r.operation_code)),
            PhotonMessage::EventData(e) => Some(("EventData", e.code)),
            PhotonMessage::InternalOperationRequest(r) => {
                Some(("InternalOperationRequest", r.operation_code))
            }
            PhotonMessage::InternalOperationResponse(r) => {
                Some(("InternalOperationResponse", r.operation_code))
            }
            _ => None,
        };

        if let Some((name, code)) = x {
            debug!("{direction} {name} {code}");
            debug!("{direction} data: {photon_message:?}");
        }

        // TODO: check if on lobby socket
        match &mut photon_message {
            PhotonMessage::EventData(event) => {
                match event.code {
                    event_code::GAME_LIST | event_code::GAME_LIST_UPDATE => {
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
                                    let mut room_info = RoomInfo::from_map(props);
                                    trace!("room {game_name}: {room_info:?}");
                                    if room_info.removed != Some(true) {
                                        if let Some(new_max_players) = spoofed_max_players {
                                            // NOTE: for some reason, max_players gets incremented by 1 ingame
                                            room_info.max_players = Some(new_max_players);
                                        }
                                    }
                                    room_info.into_map(props);
                                }
                            }
                        }
                    }
                    event_code::JOIN => {
                        let props = event.parameters.get_mut(&parameter_code::PLAYER_PROPERTIES);
                        dbg!(&props);

                        if let Some(PhotonDataType::Hashtable(props)) = props {
                            let player = Player::from_map(props);

                            info!(
                                "Received player info for {:?} (id {:?})",
                                player.nickname, player.user_id
                            );

                            player.into_map(props);
                        }
                    }
                    _ => (),
                }
            }
            // unhandled
            _ => (),
        }

        let mut buf: Vec<u8> = vec![];
        photon_message.to_websocket_bytes(&mut buf)?;
        *data = buf;

        Ok(())
    }
}
