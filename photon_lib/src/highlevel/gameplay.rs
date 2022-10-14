use crate::highlevel::constants::{actor_properties, parameter_code};
use crate::photon_data_type::PhotonDataType;
use crate::PhotonHashmap;

impl_u8_map_conversion! {
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

        /// A serialized instance of [crate::highlevel::lobby::RoomInfo]
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

    #[derive(Debug)]
    JoinGameResponse {
        [parameter_code::ROOM_NAME => PhotonDataType::String]
        room_name: String,

        [parameter_code::ACTOR_NR => PhotonDataType::Integer]
        actor_nr: i32,

        [parameter_code::ACTOR_LIST => PhotonDataType::IntArray]
        actor_list: Vec<i32>,

        /// A hashmap over serialized [Player]s
        [parameter_code::PLAYER_PROPERTIES => PhotonDataType::Hashtable]
        player_properties: PhotonHashmap,

        /// A serialized instance of [crate::highlevel::lobby::RoomInfo]
        [parameter_code::GAME_PROPERTIES => PhotonDataType::Hashtable]
        game_properties: PhotonHashmap,

        [parameter_code::ADDRESS => PhotonDataType::String]
        address: String,

        [parameter_code::ROOM_OPTION_FLAGS => PhotonDataType::Integer]
        room_option_flags: i32, // could add an impl to map this to an enum or something
    }
}

// NOTE: this macro adds a `custom_properties` field for remaining, string-keyed properties
impl_photon_map_conversion! {
    Player {
        [PhotonDataType::Byte(actor_properties::PLAYER_NAME) => PhotonDataType::String]
        nickname: String,

        [PhotonDataType::Byte(actor_properties::USER_ID) => PhotonDataType::String]
        user_id: String,

        [PhotonDataType::Byte(actor_properties::IS_INACTIVE) => PhotonDataType::Boolean]
        is_inactive: bool,
    }
}
