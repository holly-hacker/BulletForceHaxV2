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
            PhotonMessage::EventData(event) => match event.code {
                event_code::GAME_LIST | event_code::GAME_LIST_UPDATE => {
                    let show_mobile_games =
                        futures::executor::block_on(hax.lock()).show_mobile_games;
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

                                if show_mobile_games {
                                    force_games_web(&mut room_info);
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
            },
            // unhandled
            _ => (),
        }

        let mut buf: Vec<u8> = vec![];
        photon_message.to_websocket_bytes(&mut buf)?;
        *data = buf;

        Ok(())
    }
}

fn force_games_web(room_info: &mut RoomInfo) {
    let store_id = room_info.custom_properties.get("storeID").cloned();

    // adjust name if not web
    if let Some(PhotonDataType::String(name)) = room_info.custom_properties.get_mut("roomName") {
        if let Some(PhotonDataType::String(store_id)) = store_id {
            *name = match store_id.as_str() {
                "BALYZE_WEB" => name.to_string(),
                "BALYZE_MOBILE" => format!("[M] {name}"),
                v => format!("[{v}] {name}"),
            }
        }
    }

    // force game to web so it shows up in the list
    if let Some(PhotonDataType::String(x)) = room_info.custom_properties.get_mut("storeID") {
        *x = "BALYZE_WEB".into();
    }
}
