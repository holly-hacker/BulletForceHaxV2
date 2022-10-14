use std::sync::Arc;

use futures_util::lock::Mutex;
use photon_lib::{
    highlevel::{
        constants::{event_code, operation_code, parameter_code, pun_event_code},
        structs::{JoinGame, JoinGameResponse, Player, RoomInfo, RoomInfoList, RpcEvent},
        PhotonMapConversion, PhotonParameterMapConversion,
    },
    photon_data_type::PhotonDataType,
    photon_message::PhotonMessage,
};
use tracing::{debug, info, trace, warn};

use crate::{
    hax::HaxState,
    protocol::rpc::get_rpc_method_name,
    proxy::{Direction, WebSocketServer},
};

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

    pub fn websocket_hook(
        hax: Arc<Mutex<Self>>,
        data: &mut Vec<u8>,
        server: WebSocketServer,
        direction: Direction,
    ) -> anyhow::Result<()> {
        let mut photon_message = PhotonMessage::from_websocket_bytes(&mut data.as_slice())?;

        let debug_info = match &photon_message {
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

        if let Some((name, code)) = debug_info {
            debug!(name, code, direction = format!("{direction}"), "Message");

            // We're logging message_data with "full" formatting here.
            // It's a trace log which should only be logged to file and accessed in a structured
            // manner such as through json.
            trace!(
                message_type = name,
                message_code = code,
                message_data = format!("{photon_message:#?}"),
                direction = format!("{direction}"),
                "Message data"
            );
        }

        match server {
            WebSocketServer::LobbyServer => Self::match_packet_lobby(hax, &mut photon_message),
            WebSocketServer::GameServer => Self::match_packet_game(hax, &mut photon_message),
        }

        let mut buf: Vec<u8> = vec![];
        photon_message.to_websocket_bytes(&mut buf)?;
        *data = buf;

        Ok(())
    }

    fn match_packet_lobby(hax: Arc<Mutex<Self>>, photon_message: &mut PhotonMessage) {
        match photon_message {
            PhotonMessage::OperationRequest(operation_request) => {
                match operation_request.operation_code {
                    operation_code::AUTHENTICATE => {
                        let mut hax = futures::executor::block_on(hax.lock());

                        if let Some(PhotonDataType::String(app_version)) = operation_request
                            .parameters
                            .get(&parameter_code::APP_VERSION)
                        {
                            hax.game_version = Some(app_version.clone());
                        }

                        if let Some(PhotonDataType::String(user_id)) =
                            operation_request.parameters.get(&parameter_code::USER_ID)
                        {
                            hax.user_id = Some(user_id.clone());
                        }
                    }
                    _ => (),
                }
            }
            PhotonMessage::EventData(event) => match event.code {
                event_code::GAME_LIST | event_code::GAME_LIST_UPDATE => {
                    let (strip_passwords, show_mobile, show_all_versions, game_version) = {
                        let hax = futures::executor::block_on(hax.lock());
                        (
                            hax.strip_passwords,
                            hax.show_mobile_games,
                            hax.show_other_versions,
                            hax.game_version.clone(),
                        )
                    };
                    let mut game_list = RoomInfoList::from_map(&mut event.parameters);
                    if let Some(games) = &mut game_list.games {
                        for (k, v) in games.iter_mut() {
                            if let (
                                PhotonDataType::String(game_name),
                                PhotonDataType::Hashtable(props),
                            ) = (k, v)
                            {
                                let mut room_info = RoomInfo::from_map(props);

                                // NOTE: BulletForce has `gameVersion` as key so this wont match
                                if let Some(PhotonDataType::String(version)) =
                                    room_info.custom_properties.get("gameversion")
                                {
                                    if version.starts_with("newfps-") {
                                        continue;
                                    }
                                }

                                trace!("room {game_name}: {room_info:?}");

                                if show_mobile {
                                    force_games_web(&mut room_info);
                                }
                                if show_all_versions {
                                    if let Some(version) = &game_version {
                                        // versions seen in the wild are like '1.89.0_1.99', we only want the first half of that
                                        let version = match version.split_once('_') {
                                            Some((v1, _)) => v1,
                                            None => version.as_str(),
                                        };
                                        force_games_current_ver(&mut room_info, version);
                                    } else {
                                        warn!("Tried to adjust game version of lobby games but it was not known");
                                    }
                                }
                                if strip_passwords {
                                    strip_password(&mut room_info);
                                }

                                room_info.into_map(props);
                            }
                        }
                    }

                    game_list.into_map(&mut event.parameters);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn match_packet_game(hax: Arc<Mutex<Self>>, photon_message: &mut PhotonMessage) {
        match photon_message {
            PhotonMessage::OperationRequest(operation_request) => {
                match operation_request.operation_code {
                    operation_code::JOIN_GAME => {
                        let props = &mut operation_request.parameters;
                        let req = JoinGame::from_map(props);
                        debug!(request = format!("req:?"), "Game Join Request");
                        req.into_map(props);
                    }
                    _ => (),
                }
            }
            PhotonMessage::OperationResponse(operation_response) => {
                match operation_response.operation_code {
                    operation_code::JOIN_GAME if operation_response.return_code == 0 => {
                        let props = &mut operation_response.parameters;
                        let mut resp = JoinGameResponse::from_map(props);
                        debug!(response = format!("resp:?"), "Game Join Response");
                        if let Some(player_props) = &mut resp.player_properties {
                            let mut hax = futures::executor::block_on(hax.lock());

                            hax.player_id = resp.actor_nr;

                            for (key, value) in player_props {
                                let actor_id = match key {
                                    PhotonDataType::Integer(key) => *key,
                                    _ => continue,
                                };
                                let actor_props = match value {
                                    PhotonDataType::Hashtable(actor_props) => actor_props,
                                    _ => continue,
                                };
                                let actor_info = Player::from_map(&mut actor_props.clone());

                                debug!(actor_id, "Found new actor");
                                hax.players.insert(actor_id, actor_info);
                            }
                        }

                        // todo
                        resp.into_map(props);
                    }
                    _ => (),
                }
            }
            PhotonMessage::EventData(event) => match event.code {
                event_code::JOIN => {
                    let props = event.parameters.get_mut(&parameter_code::PLAYER_PROPERTIES);

                    if let Some(PhotonDataType::Hashtable(props)) = props {
                        let player = Player::from_map(props);

                        info!(
                            "Received player info for {:?} (id {:?})",
                            player.nickname, player.user_id
                        );

                        player.into_map(props);
                    }
                }
                pun_event_code::RPC => {
                    let mut event = RpcEvent::from_map(&mut event.parameters);
                    if let Some(data) = event.extract_rpc_call() {
                        let sender = data.get_owner_id();
                        let method_name = get_rpc_method_name(&data).unwrap_or_else(|_| "?".into());
                        let parameters = match &data.in_method_parameters {
                            Some(p) => p
                                .iter()
                                .map(|data| format!("{data:?}"))
                                .collect::<Vec<_>>()
                                .join(","),
                            None => String::new(),
                        };
                        debug!(
                            method_name = method_name.to_string(),
                            sender, parameters, "Incoming RPC call"
                        );
                    }
                }
                _ => (),
            },
            // unhandled
            _ => (),
        }
    }
}

fn strip_password(room_info: &mut RoomInfo) {
    let has_password = match room_info.custom_properties.get("password") {
        Some(PhotonDataType::String(s)) => !s.is_empty(),
        _ => false,
    };

    if has_password {
        if let Some(PhotonDataType::String(name)) = room_info.custom_properties.get_mut("roomName")
        {
            *name = format!("[p] {name}");
        }

        if let Some(PhotonDataType::String(password)) =
            room_info.custom_properties.get_mut("password")
        {
            *password = "".to_string();
        }
    };
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

/// Forces all games to the current version so they appear in the lobby list.
///
/// Note that this only handle BulletForce games which use `gameVersion` as key, the "newgame" game uses `gameversion`
/// (no uppercase 'v') which we dont match. This is intended.
fn force_games_current_ver(room_info: &mut RoomInfo, target_version: &str) {
    let actual_version = match room_info.custom_properties.get("gameVersion").cloned() {
        Some(PhotonDataType::String(version)) => version,
        _ => return,
    };

    if actual_version != target_version {
        if let Some(PhotonDataType::String(name)) = room_info.custom_properties.get_mut("roomName")
        {
            *name = format!("[{actual_version}] {name}");
        }

        if let Some(PhotonDataType::String(new_version)) =
            room_info.custom_properties.get_mut("gameVersion")
        {
            *new_version = target_version.to_string();
        }
    };
}
