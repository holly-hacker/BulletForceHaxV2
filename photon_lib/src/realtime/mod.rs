//! Implements high-level types PhotonRealtime.

pub mod constants;
#[macro_use]
mod macro_impl;

use indexmap::IndexMap;

use self::constants::game_property_key;
use crate::photon_data_type::PhotonDataType;

pub trait PhotonMapConversion {
    fn from_map(properties: &mut IndexMap<PhotonDataType, PhotonDataType>) -> Self;
    fn into_map(self, map: &mut IndexMap<PhotonDataType, PhotonDataType>);
}

impl_photon_map_conversion! {
    #[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use indexmap::{indexmap, IndexMap};
    use ordered_float::OrderedFloat;

    use crate::photon_data_type::PhotonDataType;
    use crate::realtime::{PhotonMapConversion, RoomInfo};

    use super::constants::game_property_key;

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
                "eventcode".into() => PhotonDataType::Integer(0),
                "timesPlayingSameMatch".into() => PhotonDataType::Integer(1),
                "averagerank".into() => PhotonDataType::Integer(14),
                "dedicated".into() => PhotonDataType::Boolean(false),
                "roomType".into() => PhotonDataType::Byte(0),
                "password".into() => PhotonDataType::String("".into()),
                "gameVersion".into() => PhotonDataType::String("1.89.1".into()),
                "isDevMatch".into() => PhotonDataType::Boolean(false),
                "meanRank".into() => PhotonDataType::Float(OrderedFloat(4.6666665)),
                "gameID".into() => PhotonDataType::String("PC-leys2006_1665094204".into()),
                "mapName".into() => PhotonDataType::String("Woods".into()),
                "matchStarted".into() => PhotonDataType::Boolean(true),
                "roomID".into() => PhotonDataType::String("PC-leys2006_1665094175".into()),
                "storeID".into() => PhotonDataType::String("BALYZE_WEB".into()),
                "modeName".into() => PhotonDataType::String("Team Deathmatch".into()),
                "allowedweapons".into() => PhotonDataType::Array(vec![PhotonDataType::Integer(-1),PhotonDataType::Integer(-1),PhotonDataType::Integer(-1)]),
                "roomName".into() => PhotonDataType::String("Beginner (#5625)".into()),
                "hardcore".into() => PhotonDataType::Boolean(false)
            },
        };

        let photon_map = indexmap! {
            PhotonDataType::Byte(game_property_key::MAX_PLAYERS) => PhotonDataType::Byte(15),
            PhotonDataType::Byte(game_property_key::IS_OPEN) => PhotonDataType::Boolean(true),
            PhotonDataType::Byte(game_property_key::PLAYER_COUNT) => PhotonDataType::Byte(3),
            PhotonDataType::String("switchingmap".into()) => PhotonDataType::Boolean(false),
            PhotonDataType::String("meanKD".into()) => PhotonDataType::Float(OrderedFloat(0.72795415)),
            PhotonDataType::String("seasonID".into()) => PhotonDataType::String("".into()),
            PhotonDataType::String("eventcode".into()) => PhotonDataType::Integer(0),
            PhotonDataType::String("timesPlayingSameMatch".into()) => PhotonDataType::Integer(1),
            PhotonDataType::String("averagerank".into()) => PhotonDataType::Integer(14),
            PhotonDataType::String("dedicated".into()) => PhotonDataType::Boolean(false),
            PhotonDataType::String("roomType".into()) => PhotonDataType::Byte(0),
            PhotonDataType::String("password".into()) => PhotonDataType::String("".into()),
            PhotonDataType::String("gameVersion".into()) => PhotonDataType::String("1.89.1".into()),
            PhotonDataType::String("isDevMatch".into()) => PhotonDataType::Boolean(false),
            PhotonDataType::String("meanRank".into()) => PhotonDataType::Float(OrderedFloat(4.6666665)),
            PhotonDataType::String("gameID".into()) => PhotonDataType::String("PC-leys2006_1665094204".into()),
            PhotonDataType::String("mapName".into()) => PhotonDataType::String("Woods".into()),
            PhotonDataType::String("matchStarted".into()) => PhotonDataType::Boolean(true),
            PhotonDataType::String("roomID".into()) => PhotonDataType::String("PC-leys2006_1665094175".into()),
            PhotonDataType::String("storeID".into()) => PhotonDataType::String("BALYZE_WEB".into()),
            PhotonDataType::String("modeName".into()) => PhotonDataType::String("Team Deathmatch".into()),
            PhotonDataType::String("allowedweapons".into()) => PhotonDataType::Array(vec![PhotonDataType::Integer(-1),PhotonDataType::Integer(-1),PhotonDataType::Integer(-1)]),
            PhotonDataType::String("roomName".into()) => PhotonDataType::String("Beginner (#5625)".into()),
            PhotonDataType::String("hardcore".into()) => PhotonDataType::Boolean(false),
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
