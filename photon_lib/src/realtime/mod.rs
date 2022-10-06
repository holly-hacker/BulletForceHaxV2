//! Implements high-level types PhotonRealtime.

pub mod constants;
#[macro_use]
mod macro_impl;

use indexmap::IndexMap;

use self::constants::game_property_key;
use crate::photon_data_type::PhotonDataType;

pub trait EventDataBased {
    fn from_hashtable(properties: &mut IndexMap<PhotonDataType, PhotonDataType>) -> Self;
    fn into_hashtable(self, map: &mut IndexMap<PhotonDataType, PhotonDataType>);
}

impl_hashtable! {
    RoomInfo {
        #[PhotonDataType::Byte(game_property_key::REMOVED) => PhotonDataType::Boolean]
        removed: bool,

        #[PhotonDataType::Byte(game_property_key::MAX_PLAYERS) => PhotonDataType::Byte]
        max_players: u8,

        #[PhotonDataType::Byte(game_property_key::IS_OPEN) => PhotonDataType::Boolean]
        is_open: bool,

        #[PhotonDataType::Byte(game_property_key::IS_VISIBLE) => PhotonDataType::Boolean]
        is_visible: bool,

        #[PhotonDataType::Byte(game_property_key::PLAYER_COUNT) => PhotonDataType::Byte]
        player_count: u8,

        #[PhotonDataType::Byte(game_property_key::CLEANUP_CACHE_ON_LEAVE) => PhotonDataType::Boolean]
        cleanup_cache_on_leave: bool,

        #[PhotonDataType::Byte(game_property_key::MASTER_CLIENT_ID) => PhotonDataType::Integer]
        master_client_id: i32,

        #[PhotonDataType::Byte(game_property_key::PROPS_LISTED_IN_LOBBY) => PhotonDataType::StringArray]
        props_listed_in_lobby: Vec<String>,

        #[PhotonDataType::Byte(game_property_key::EXPECTED_USERS) => PhotonDataType::StringArray]
        expected_users: Vec<String>,

        #[PhotonDataType::Byte(game_property_key::EMPTY_ROOM_TTL) => PhotonDataType::Integer]
        empty_room_ttl: i32,

        #[PhotonDataType::Byte(game_property_key::PLAYER_TTL) => PhotonDataType::Integer]
        player_ttl: i32,
    }
}
