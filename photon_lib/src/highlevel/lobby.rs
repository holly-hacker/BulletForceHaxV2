use crate::highlevel::constants::{game_property_key, parameter_code};
use crate::photon_data_type::PhotonDataType;
use crate::PhotonHashmap;

impl_u8_map_conversion! {
    RoomInfoList {
        [parameter_code::GAME_LIST => PhotonDataType::Hashtable]
        games: PhotonHashmap,
    }
}

// NOTE: this macro adds a `custom_properties` field for remaining, string-keyed properties
impl_photon_map_conversion! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    RoomInfo {
        [PhotonDataType::Byte(game_property_key::REMOVED) => PhotonDataType::Boolean]
        removed: bool,

        [PhotonDataType::Byte(game_property_key::MAX_PLAYERS) => PhotonDataType::Byte]
        max_players: u8,

        [PhotonDataType::Byte(game_property_key::IS_OPEN) => PhotonDataType::Boolean]
        is_open: bool,

        [PhotonDataType::Byte(game_property_key::IS_VISIBLE) => PhotonDataType::Boolean]
        is_visible: bool,

        [PhotonDataType::Byte(game_property_key::PLAYER_COUNT) => PhotonDataType::Byte]
        player_count: u8,

        [PhotonDataType::Byte(game_property_key::CLEANUP_CACHE_ON_LEAVE) => PhotonDataType::Boolean]
        cleanup_cache_on_leave: bool,

        [PhotonDataType::Byte(game_property_key::MASTER_CLIENT_ID) => PhotonDataType::Integer]
        master_client_id: i32,

        [PhotonDataType::Byte(game_property_key::PROPS_LISTED_IN_LOBBY) => PhotonDataType::StringArray]
        props_listed_in_lobby: Vec<String>,

        [PhotonDataType::Byte(game_property_key::EXPECTED_USERS) => PhotonDataType::StringArray]
        expected_users: Vec<String>,

        [PhotonDataType::Byte(game_property_key::EMPTY_ROOM_TTL) => PhotonDataType::Integer]
        empty_room_ttl: i32,

        [PhotonDataType::Byte(game_property_key::PLAYER_TTL) => PhotonDataType::Integer]
        player_ttl: i32,
    }
}

#[cfg(test)]
mod tests {
    use indexmap::{indexmap, IndexMap};
    use ordered_float::OrderedFloat;

    use crate::highlevel::constants::game_property_key;
    use crate::highlevel::lobby::RoomInfo;
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
