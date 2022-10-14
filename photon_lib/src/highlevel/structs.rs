//! High-level representations of photon messages.

pub use super::structs_impl::*;
use crate::highlevel::constants::{actor_properties, game_property_key, parameter_code};
#[allow(unused)]
use crate::highlevel::constants::{event_code, operation_code, pun_event_code};
use crate::photon_data_type::PhotonDataType;
use crate::PhotonHashmap;

impl_u8_map_conversion! {
    /// Parameter of [event_code::GAME_LIST] and [event_code::GAME_LIST_UPDATE]. Contains a list of [RoomInfo].
    RoomInfoList {
        [parameter_code::GAME_LIST => PhotonDataType::Hashtable]
        games: PhotonHashmap,
    }

    /// Request parameter of [operation_code::JOIN_GAME].
    #[derive(Debug)]
    JoinGame {
        [parameter_code::ROOM_NAME => PhotonDataType::String]
        room_name: String,

        [parameter_code::PROPERTIES => PhotonDataType::Hashtable]
        properties: PhotonHashmap,

        [parameter_code::BROADCAST => PhotonDataType::Boolean]
        broadcast: bool,

        [parameter_code::PLAYER_PROPERTIES => PhotonDataType::Hashtable]
        player_properties: PhotonHashmap,

        /// A serialized instance of [RoomInfo]
        [parameter_code::GAME_PROPERTIES => PhotonDataType::Hashtable]
        game_properties: PhotonHashmap,

        [parameter_code::CLEANUP_CACHE_ON_LEAVE => PhotonDataType::Boolean]
        cleanup_cache_on_leave: bool,

        [parameter_code::PUBLISH_USER_ID => PhotonDataType::Boolean]
        publis_user_id: bool,

        [parameter_code::ADD => PhotonDataType::StringArray]
        add: Vec<String>,

        [parameter_code::SUPPRESS_ROOM_EVENTS => PhotonDataType::Boolean]
        suppress_room_events: bool,

        [parameter_code::EMPTY_ROOM_TTL => PhotonDataType::Integer]
        empty_room_ttl: i32,

        [parameter_code::PLAYER_TTL => PhotonDataType::Integer]
        player_ttl: i32,

        [parameter_code::CHECK_USER_ON_JOIN => PhotonDataType::Boolean]
        check_user_on_join: bool,

        [parameter_code::JOIN_MODE => PhotonDataType::Byte]
        join_mode: u8,

        [parameter_code::LOBBY_NAME => PhotonDataType::String]
        lobby_name: String,

        [parameter_code::LOBBY_TYPE => PhotonDataType::Byte]
        lobby_type: u8,

        [parameter_code::PLUGINS => PhotonDataType::StringArray]
        plugins: Vec<String>,

        [parameter_code::ROOM_OPTION_FLAGS => PhotonDataType::Integer]
        room_option_flags: i32, // could add an impl to map this to an enum or something
    }

    /// Response parameter of [operation_code::JOIN_GAME].
    #[derive(Debug)]
    JoinGameResponse {
        [parameter_code::ROOM_NAME => PhotonDataType::String]
        room_name: String,

        [parameter_code::ACTOR_NR => PhotonDataType::Integer]
        actor_nr: i32,

        [parameter_code::ACTOR_LIST => PhotonDataType::IntArray]
        actor_list: Vec<i32>,

        /// A hashmap over serialized [Player]s. The keys in this hashmap are integer actor ids.
        [parameter_code::PLAYER_PROPERTIES => PhotonDataType::Hashtable]
        player_properties: PhotonHashmap,

        /// A serialized instance of [RoomInfo]
        [parameter_code::GAME_PROPERTIES => PhotonDataType::Hashtable]
        game_properties: PhotonHashmap,

        [parameter_code::ADDRESS => PhotonDataType::String]
        address: String,

        [parameter_code::ROOM_OPTION_FLAGS => PhotonDataType::Integer]
        room_option_flags: i32, // could add an impl to map this to an enum or something
    }

    /// Parameter for [pun_event_code::RPC]. Contains a single [RpcCall].
    #[derive(Debug)]
    RpcEvent {
        [parameter_code::ACTOR_NR => PhotonDataType::Integer]
        sender_actor: i32,

        [parameter_code::CUSTOM_EVENT_CONTENT => PhotonDataType::Hashtable]
        data: PhotonHashmap,
    }
}

// NOTE: this macro adds a `custom_properties` field for remaining, string-keyed properties
impl_photon_map_conversion! {
    /// Describes a room.
    #[derive(Debug, Clone, PartialEq, Eq)]
    RoomInfo {
        /// If `true`, this game should be removed from the game list in the lobby. Not used during gameplay.
        [PhotonDataType::Byte(game_property_key::REMOVED) => PhotonDataType::Boolean]
        removed: bool,

        /// Indicates how many players can be in this room. 0 means no limit.
        [PhotonDataType::Byte(game_property_key::MAX_PLAYERS) => PhotonDataType::Byte]
        max_players: u8,

        /// Indicates if the room can be joined.
        [PhotonDataType::Byte(game_property_key::IS_OPEN) => PhotonDataType::Boolean]
        is_open: bool,

        /// Indicates if this room should be shown in the lobby. Invisible rooms can still be joined.
        [PhotonDataType::Byte(game_property_key::IS_VISIBLE) => PhotonDataType::Boolean]
        is_visible: bool,

        [PhotonDataType::Byte(game_property_key::PLAYER_COUNT) => PhotonDataType::Byte]
        player_count: u8,

        [PhotonDataType::Byte(game_property_key::CLEANUP_CACHE_ON_LEAVE) => PhotonDataType::Boolean]
        cleanup_cache_on_leave: bool,

        /// The actor id of the master client.
        [PhotonDataType::Byte(game_property_key::MASTER_CLIENT_ID) => PhotonDataType::Integer]
        master_client_id: i32,

        [PhotonDataType::Byte(game_property_key::PROPS_LISTED_IN_LOBBY) => PhotonDataType::StringArray]
        props_listed_in_lobby: Vec<String>,

        /// Instructs the server to keep player slots open for these players.
        [PhotonDataType::Byte(game_property_key::EXPECTED_USERS) => PhotonDataType::StringArray]
        expected_users: Vec<String>,

        /// How long the room stays alive after the last player left.
        ///
        /// See also [RoomInfo::player_ttl].
        [PhotonDataType::Byte(game_property_key::EMPTY_ROOM_TTL) => PhotonDataType::Integer]
        empty_room_ttl: i32,

        /// How long a player stays "active" after disconnecting. As long as this time has not passed, their slot stays occupied.
        ///
        /// See also [Player::is_inactive].
        [PhotonDataType::Byte(game_property_key::PLAYER_TTL) => PhotonDataType::Integer]
        player_ttl: i32,
    }

    /// Describes a player. Most information wil be in [Player::custom_properties].
    Player {
        // The player's nickname.
        [PhotonDataType::Byte(actor_properties::PLAYER_NAME) => PhotonDataType::String]
        nickname: String,

        /// Not always present.
        [PhotonDataType::Byte(actor_properties::USER_ID) => PhotonDataType::String]
        user_id: String,

        [PhotonDataType::Byte(actor_properties::IS_INACTIVE) => PhotonDataType::Boolean]
        is_inactive: bool,
    }

    /// An RPC call. Can be both sent and received by the client.
    RpcCall {
        [PhotonDataType::Byte(0) => PhotonDataType::Integer]
        @net_view_id: i32,

        [PhotonDataType::Byte(1) => PhotonDataType::Short]
        other_side_prefix: i16,

        /// Mutually exclusive with [RpcCall::rpc_index]
        [PhotonDataType::Byte(3) => PhotonDataType::String]
        method_name: String,

        [PhotonDataType::Byte(4) => PhotonDataType::ObjectArray]
        in_method_parameters: Vec<PhotonDataType>,

        /// Mutually exclusive with [RpcCall::method_name]
        [PhotonDataType::Byte(5) => PhotonDataType::Byte]
        rpc_index: u8,
    }
}

#[cfg(test)]
mod tests {
    use indexmap::{indexmap, IndexMap};
    use ordered_float::OrderedFloat;

    use super::RoomInfo;
    use crate::highlevel::constants::game_property_key;
    use crate::highlevel::PhotonMapConversion;
    use crate::photon_data_type::PhotonDataType;

    #[test]
    fn room_info() {
        let room_info = RoomInfo {
            removed: None,
            max_players: Some(15),
            is_open: Some(true),
            is_visible: None,
            player_count: Some(3),
            cleanup_cache_on_leave: None,
            master_client_id: None,
            props_listed_in_lobby: None,
            expected_users: None,
            empty_room_ttl: None,
            player_ttl: None,
            custom_properties: indexmap! {
                "switchingmap".into() => PhotonDataType::Boolean(false),
                "meanKD".into() => PhotonDataType::Float(OrderedFloat(0.72795415)),
                "seasonID".into() => PhotonDataType::String("".into()),
                "eventcode".into() => PhotonDataType::Integer(0)
            },
        };

        let photon_map = indexmap! {
            PhotonDataType::String("switchingmap".into()) => PhotonDataType::Boolean(false),
            PhotonDataType::Byte(game_property_key::MAX_PLAYERS) => PhotonDataType::Byte(15),
            PhotonDataType::String("meanKD".into()) => PhotonDataType::Float(OrderedFloat(0.72795415)),
            PhotonDataType::Byte(game_property_key::IS_OPEN) => PhotonDataType::Boolean(true),
            PhotonDataType::String("seasonID".into()) => PhotonDataType::String("".into()),
            PhotonDataType::Byte(game_property_key::PLAYER_COUNT) => PhotonDataType::Byte(3),
            PhotonDataType::String("eventcode".into()) => PhotonDataType::Integer(0)
        };

        {
            let mut deserialized = IndexMap::new();
            room_info.clone().into_map(&mut deserialized);
            assert_eq!(deserialized, photon_map);
        }

        {
            let serialized = RoomInfo::from_map(&mut photon_map.clone());
            assert_eq!(serialized, room_info);
        }
    }
}
