use std::{ops::DerefMut, sync::Arc};

use futures_util::lock::Mutex;
use photon_lib::{
    highlevel::{
        constants::{event_code, operation_code, parameter_code, pun_event_code},
        structs::{
            DestroyEvent, DestroyEventData, InstantiationEvent, InstantiationEventData,
            JoinGameRequest, JoinGameResponseSuccess, LeaveEvent, Player, PropertiesChangedEvent,
            RaiseEvent, RoomInfo, RoomInfoList, RpcCall, RpcEvent, SendSerializeEvent,
            SetPropertiesOperationRequest,
        },
        PhotonMapConversion, PhotonParameterMapConversion,
    },
    photon_data_type::PhotonDataType,
    photon_message::PhotonMessage,
};
use tracing::{debug, trace, warn};

use super::VersionInfo;
use crate::{
    hax::{HaxState, PlayerActor},
    protocol::{player_script::PlayerScript, rpc::get_rpc_method_name},
    proxy::{Direction, WebSocketServer},
};

#[allow(dead_code)]
enum WebSocketHookAction {
    /// Replace the original message with the given one
    Change(PhotonMessage),
    /// Drop this message completely, don't forward it to the client/server
    Drop,
    /// Do nothing, just pass along the original message
    DoNothing,
}

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

    /// Runs logic on this websocket message and returns whether the given data should be forwarded on.
    pub fn websocket_hook(
        hax: Arc<Mutex<Self>>,
        data: &mut Vec<u8>,
        server: WebSocketServer,
        direction: Direction,
    ) -> anyhow::Result<bool> {
        let photon_message = PhotonMessage::from_websocket_bytes(&mut data.as_slice())?;

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

        let action = match server {
            WebSocketServer::LobbyServer => Self::match_packet_lobby(hax, photon_message)?,
            WebSocketServer::GameServer => Self::match_packet_game(hax, photon_message)?,
        };

        match action {
            WebSocketHookAction::Change(new_message) => {
                let mut buf: Vec<u8> = vec![];
                new_message.to_websocket_bytes(&mut buf)?;
                *data = buf;
            }
            WebSocketHookAction::Drop => return Ok(false),
            WebSocketHookAction::DoNothing => (),
        }

        Ok(true)
    }

    fn match_packet_lobby(
        hax: Arc<Mutex<Self>>,
        photon_message: PhotonMessage,
    ) -> anyhow::Result<WebSocketHookAction> {
        match photon_message {
            PhotonMessage::OperationRequest(operation_request) => {
                match operation_request.operation_code {
                    operation_code::AUTHENTICATE => {
                        let mut hax = futures::executor::block_on(hax.lock());

                        if let Some(PhotonDataType::String(app_version)) = operation_request
                            .parameters
                            .get(&parameter_code::APP_VERSION)
                        {
                            let version = app_version.split_once('_');
                            hax.global_state.version = version.map(|(game, photon)| VersionInfo {
                                game_version: game.to_string(),
                                photon_version: photon.to_string(),
                            });
                        }

                        if let Some(PhotonDataType::String(user_id)) =
                            operation_request.parameters.get(&parameter_code::USER_ID)
                        {
                            hax.global_state.user_id = Some(user_id.clone());
                        }
                    }
                    _ => (),
                }
            }
            PhotonMessage::EventData(mut event) => match event.code {
                event_code::GAME_LIST | event_code::GAME_LIST_UPDATE => {
                    let (strip_passwords, show_mobile, show_all_versions, game_version) = {
                        let hax = futures::executor::block_on(hax.lock());
                        (
                            hax.strip_passwords,
                            hax.show_mobile_games,
                            hax.show_other_versions,
                            hax.global_state.version.clone(),
                        )
                    };
                    let mut game_list = RoomInfoList::from_map(&mut event.parameters)?;
                    let mut changes_made = false;

                    for (k, v) in game_list.games.iter_mut() {
                        if let (
                            PhotonDataType::String(game_name),
                            PhotonDataType::Hashtable(props),
                        ) = (k, v)
                        {
                            let mut room_info = RoomInfo::from_map(props)?;

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
                                changes_made = true;
                            }
                            if show_all_versions {
                                if let Some(version) = &game_version {
                                    force_games_current_ver(&mut room_info, &version.game_version);
                                    changes_made = true;
                                } else {
                                    warn!("Tried to adjust game version of lobby games but it was not known");
                                }
                            }
                            if strip_passwords {
                                strip_password(&mut room_info);
                                changes_made = true;
                            }

                            room_info.into_map(props);
                        }
                    }

                    // prevent doing work if we didnt actually change anything
                    if changes_made {
                        game_list.into_map(&mut event.parameters);
                        return Ok(WebSocketHookAction::Change(PhotonMessage::EventData(event)));
                    }
                }
                _ => (),
            },
            _ => (),
        }

        Ok(WebSocketHookAction::DoNothing)
    }

    fn match_packet_game(
        hax: Arc<Mutex<Self>>,
        photon_message: PhotonMessage,
    ) -> anyhow::Result<WebSocketHookAction> {
        match photon_message {
            PhotonMessage::OperationRequest(mut operation_request) => {
                match operation_request.operation_code {
                    operation_code::JOIN_GAME => {
                        let props = &mut operation_request.parameters;
                        let _req = JoinGameRequest::from_map(props)?;
                        debug!(request = format!("_req:?"), "Game Join Request");
                    }

                    operation_code::SET_PROPERTIES => {
                        let mut req = SetPropertiesOperationRequest::from_map(
                            &mut operation_request.parameters,
                        )?;

                        if let Some(actor) = req.actor_nr {
                            // properties are for actor, not for room
                            let player_props = Player::from_map(&mut req.properties)?;

                            let mut hax = futures::executor::block_on(hax.lock());
                            let (_, state) = match &mut hax.gameplay_state {
                                Some(x) => x,
                                _ => anyhow::bail!("gameplay state is None"),
                            };

                            if let Some(player) = state.players.get_mut(&actor) {
                                player.merge_player(player_props);
                            }
                        }
                    }

                    operation_code::RAISE_EVENT => {
                        let req = RaiseEvent::from_map(&mut operation_request.parameters)?;

                        debug!(
                            event_code = req.event_code,
                            data = format!("{:?}", req.data),
                            "Raise event"
                        );

                        let req_data = match req.data {
                            Some(PhotonDataType::Hashtable(t)) => Some(t),
                            _ => None,
                        };

                        match req.event_code {
                            event_code::LEAVE
                            | event_code::PROPERTIES_CHANGED
                            | pun_event_code::DESTROY => {
                                // never seen these happen in practice. If they occur, document them!
                                warn!(event_code = req.event_code, "Unexpected raised event");
                            }
                            pun_event_code::INSTANTIATION => {
                                let mut req_data = match req_data {
                                    Some(x) => x,
                                    None => anyhow::bail!("INSTANTIATION event without data"),
                                };

                                let event_data = InstantiationEventData::from_map(&mut req_data)?;
                                let sender = event_data.get_view_id().get_owner_id();
                                debug!(
                                    data = format!("{event_data:?}"),
                                    sender,
                                    direction = "server",
                                    "Instantiation"
                                );

                                let hax = futures::executor::block_on(hax.lock());
                                merge_instantiation(hax, sender, &event_data)?;
                            }
                            pun_event_code::SEND_SERIALIZE
                            | pun_event_code::SEND_SERIALIZE_RELIABLE => {
                                let req_data = match req_data {
                                    Some(x) => x,
                                    None => anyhow::bail!(
                                        "SEND_SERIALIZE(_RELIABLE) event without data"
                                    ),
                                };
                                let serialized_data =
                                    SendSerializeEvent::parse_serialized_data(&req_data)
                                        .ok_or_else(|| {
                                            anyhow::anyhow!("Cound not parse serialized data")
                                        })?;

                                let mut hax = futures::executor::block_on(hax.lock());
                                let (_, state) = match &mut hax.gameplay_state {
                                    Some(x) => x,
                                    _ => anyhow::bail!("gameplay state is None"),
                                };

                                for obj in serialized_data {
                                    let actor_id = obj.get_view_id().get_owner_id();
                                    if let Some(actor) = state.players.get_mut(&actor_id) {
                                        let player_script =
                                            PlayerScript::from_object_array(&obj.data_stream)?;
                                        trace!(
                                            actor_id,
                                            player_script = format!("{player_script:?}"),
                                            "SendSerialize for actor"
                                        );

                                        actor.merge_player_script(&player_script);
                                    }
                                    trace!(
                                        direction = "client",
                                        view_id = obj.view_id,
                                        data = format!("{:?}", obj.data_stream),
                                        "SendSerialize"
                                    );
                                }
                            }
                            pun_event_code::RPC => {
                                // client->server RPC call
                                let mut event_content = req_data
                                    .ok_or_else(|| anyhow::anyhow!("RPC call with no data"))?;

                                let data = RpcCall::from_map(&mut event_content)?;

                                let sender = data.get_view_id().get_owner_id();
                                let method_name =
                                    get_rpc_method_name(&data).unwrap_or_else(|_| "?".into());
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
                                    sender,
                                    parameters,
                                    direction = "server",
                                    "RPC call"
                                );
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            PhotonMessage::OperationResponse(mut operation_response) => {
                match operation_response.operation_code {
                    operation_code::JOIN_GAME if operation_response.return_code == 0 => {
                        let props = &mut operation_response.parameters;
                        let mut resp = JoinGameResponseSuccess::from_map(props)?;
                        debug!(response = format!("resp:?"), "Game Join Response");
                        let mut hax = futures::executor::block_on(hax.lock());
                        let (_, state) = match &mut hax.gameplay_state {
                            Some(x) => x,
                            _ => anyhow::bail!("gameplay state is None"),
                        };

                        state.player_id = Some(resp.actor_nr);

                        for (key, value) in &mut resp.player_properties {
                            let actor_id = match key {
                                PhotonDataType::Integer(key) => *key,
                                _ => continue,
                            };
                            let actor_props = match value {
                                PhotonDataType::Hashtable(actor_props) => actor_props,
                                _ => continue,
                            };

                            let mut actor = PlayerActor::default();

                            let player = Player::from_map(&mut actor_props.clone())?;
                            actor.merge_player(player);

                            debug!(actor_id, "Found new actor");
                            state.players.insert(actor_id, actor);
                        }

                        tracing::info!(
                            players = format!("{:?}", state.players),
                            "Player info after join"
                        );
                    }
                    _ => (),
                }
            }
            PhotonMessage::EventData(mut event) => match event.code {
                event_code::JOIN => {
                    let mut hax = futures::executor::block_on(hax.lock());
                    let (_, state) = match &mut hax.gameplay_state {
                        Some(x) => x,
                        _ => anyhow::bail!("gameplay state is None"),
                    };

                    let actor_nr = event.parameters.get_mut(&parameter_code::ACTOR_NR);
                    if let Some(PhotonDataType::Integer(actor_nr)) = actor_nr {
                        state.actor_nr = Some(*actor_nr);
                    }

                    let actor_list = event.parameters.get_mut(&parameter_code::ACTOR_LIST);
                    if let Some(PhotonDataType::Array(array)) = actor_list {
                        for id in array {
                            if let PhotonDataType::Integer(id) = id {
                                state
                                    .players
                                    .entry(*id)
                                    .or_insert_with(PlayerActor::default);
                            }
                        }
                    }

                    // PLAYER_PROPERTIES field is pretty useless, only contains empty string as nickname
                }
                event_code::LEAVE => {
                    let event = LeaveEvent::from_map(&mut event.parameters)?;
                    let sender = event.sender_actor.unwrap_or(-1);
                    debug!(
                        data = format!("{event:?}"),
                        sender,
                        direction = "client",
                        "Leave"
                    );

                    let mut hax = futures::executor::block_on(hax.lock());
                    let (_, state) = match &mut hax.gameplay_state {
                        Some(x) => x,
                        _ => anyhow::bail!("gameplay state is None"),
                    };

                    state.players.remove(&sender);
                }
                event_code::PROPERTIES_CHANGED => {
                    let mut event = PropertiesChangedEvent::from_map(&mut event.parameters)?;
                    let sender = event.sender_actor.unwrap_or(-1);
                    debug!(
                        data = format!("{event:?}"),
                        sender,
                        direction = "client",
                        "PropertiesChanged"
                    );

                    if event.target_actor_number != 0 {
                        let mut hax = futures::executor::block_on(hax.lock());
                        let (_, state) = match &mut hax.gameplay_state {
                            Some(x) => x,
                            _ => anyhow::bail!("gameplay state is None"),
                        };

                        let player = state
                            .players
                            .get_mut(&event.target_actor_number)
                            .ok_or_else(|| anyhow::anyhow!("Failed to find actor"))?;

                        let player_props = Player::from_map(&mut event.properties)?;

                        player.merge_player(player_props);
                    }
                }
                // NOTE: this only destroys the game object
                pun_event_code::DESTROY => {
                    let mut event = DestroyEvent::from_map(&mut event.parameters)?;
                    let sender = event.sender_actor.unwrap_or(-1);
                    let event_data = DestroyEventData::from_map(&mut event.data)?;
                    debug!(
                        data = format!("{event_data:?}"),
                        sender,
                        direction = "client",
                        "Destroy"
                    );
                }
                pun_event_code::INSTANTIATION => {
                    let mut event = InstantiationEvent::from_map(&mut event.parameters)?;
                    let sender = event.sender_actor.unwrap_or(-1);
                    let event_data = InstantiationEventData::from_map(&mut event.data)?;
                    debug!(
                        data = format!("{event_data:?}"),
                        sender,
                        direction = "client",
                        "Instantiation"
                    );

                    let hax = futures::executor::block_on(hax.lock());
                    merge_instantiation(hax, sender, &event_data)?;
                }
                pun_event_code::SEND_SERIALIZE | pun_event_code::SEND_SERIALIZE_RELIABLE => {
                    let event = SendSerializeEvent::from_map(&mut event.parameters)?;
                    let serialized_data = event
                        .get_serialized_data()
                        .ok_or_else(|| anyhow::anyhow!("SendSerialize data error"))?;

                    let mut hax = futures::executor::block_on(hax.lock());
                    let (_, state) = match &mut hax.gameplay_state {
                        Some(x) => x,
                        _ => anyhow::bail!("gameplay state is None"),
                    };

                    for obj in serialized_data {
                        let actor_id = obj.get_view_id().get_owner_id();
                        if let Some(actor) = state.players.get_mut(&actor_id) {
                            let player_script = PlayerScript::from_object_array(&obj.data_stream)?;
                            trace!(
                                actor_id,
                                player_script = format!("{player_script:?}"),
                                "SendSerialize for actor"
                            );

                            actor.merge_player_script(&player_script);
                        }
                        trace!(
                            direction = "client",
                            view_id = obj.view_id,
                            data = format!("{:?}", obj.data_stream),
                            "SendSerialize"
                        );
                    }
                }
                pun_event_code::RPC => {
                    let mut event = RpcEvent::from_map(&mut event.parameters)?;
                    let data = event.extract_rpc_call()?;

                    let sender = data.get_view_id().get_owner_id();
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
                        sender,
                        parameters,
                        direction = "client",
                        "RPC call"
                    );
                }
                _ => (),
            },
            // unhandled
            _ => (),
        }

        Ok(WebSocketHookAction::DoNothing)
    }
}

fn merge_instantiation(
    mut hax: impl DerefMut<Target = HaxState>,
    sender: i32,
    event_data: &InstantiationEventData,
) -> anyhow::Result<()> {
    let (_, state) = match &mut hax.gameplay_state {
        Some(x) => x,
        _ => anyhow::bail!("gameplay state is None"),
    };

    match event_data.prefab_name.as_ref() {
        "PlayerBody" => {
            let x = state.players.entry(sender).or_default();
            x.merge_instantiation_data(event_data);
        }
        "Match Manager" => {
            state.match_manager_view_id = Some(event_data.instantiation_id);
        }
        n => debug!(name = n, "Unknown prefab name in instantiation packet"),
    }

    Ok(())
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
