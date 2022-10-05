//! Implements high-level types PhotonRealtime.

pub mod constants;

use indexmap::IndexMap;
use tracing::warn;

use crate::photon_data_type::PhotonDataType;

use self::constants::game_property_key;

pub trait EventDataBased {
    fn from_hashtable(properties: &mut IndexMap<PhotonDataType, PhotonDataType>) -> Self;
    fn into_hashtable(self, map: &mut IndexMap<PhotonDataType, PhotonDataType>);
}

#[derive(Debug)]
pub struct RoomInfo {
    pub removed: Option<bool>,
    pub max_players: Option<u8>,
    pub is_open: Option<bool>,
    pub is_visible: Option<bool>,
    pub player_count: Option<u8>,
    pub cleanup_cache_on_leave: Option<bool>,
    pub master_client_id: Option<i32>,
    pub props_listed_in_lobby: Option<Vec<String>>,
    pub expected_users: Option<Vec<String>>,
    pub empty_room_ttl: Option<i32>,
    pub player_ttl: Option<i32>,
    pub custom_properties: IndexMap<String, PhotonDataType>,
}

impl EventDataBased for RoomInfo {
    fn from_hashtable(properties: &mut IndexMap<PhotonDataType, PhotonDataType>) -> Self {
        RoomInfo {
            removed: match properties.remove(&PhotonDataType::Byte(game_property_key::REMOVED)) {
                Some(PhotonDataType::Boolean(b)) => Some(b),
                Some(_) => {
                    warn!("Unexpected data type");
                    None
                }
                _ => None,
            },
            max_players: match properties
                .remove(&PhotonDataType::Byte(game_property_key::MAX_PLAYERS))
            {
                Some(PhotonDataType::Byte(b)) => Some(b),
                Some(_) => {
                    warn!("Unexpected data type");
                    None
                }
                _ => None,
            },
            is_open: match properties.remove(&PhotonDataType::Byte(game_property_key::IS_OPEN)) {
                Some(PhotonDataType::Boolean(b)) => Some(b),
                Some(_) => {
                    warn!("Unexpected data type");
                    None
                }
                _ => None,
            },
            is_visible: match properties
                .remove(&PhotonDataType::Byte(game_property_key::IS_VISIBLE))
            {
                Some(PhotonDataType::Boolean(b)) => Some(b),
                Some(_) => {
                    warn!("Unexpected data type");
                    None
                }
                _ => None,
            },
            player_count: match properties
                .remove(&PhotonDataType::Byte(game_property_key::PLAYER_COUNT))
            {
                Some(PhotonDataType::Byte(b)) => Some(b),
                Some(_) => {
                    warn!("Unexpected data type");
                    None
                }
                _ => None,
            },
            cleanup_cache_on_leave: match properties.remove(&PhotonDataType::Byte(
                game_property_key::CLEANUP_CACHE_ON_LEAVE,
            )) {
                Some(PhotonDataType::Boolean(b)) => Some(b),
                Some(_) => {
                    warn!("Unexpected data type");
                    None
                }
                _ => None,
            },
            master_client_id: match properties
                .remove(&PhotonDataType::Byte(game_property_key::MASTER_CLIENT_ID))
            {
                Some(PhotonDataType::Integer(b)) => Some(b),
                Some(_) => {
                    warn!("Unexpected data type");
                    None
                }
                _ => None,
            },
            props_listed_in_lobby: match properties.remove(&PhotonDataType::Byte(
                game_property_key::PROPS_LISTED_IN_LOBBY,
            )) {
                Some(PhotonDataType::StringArray(b)) => Some(b),
                Some(_) => {
                    warn!("Unexpected data type");
                    None
                }
                _ => None,
            },
            expected_users: match properties
                .remove(&PhotonDataType::Byte(game_property_key::EXPECTED_USERS))
            {
                Some(PhotonDataType::StringArray(b)) => Some(b),
                Some(_) => {
                    warn!("Unexpected data type");
                    None
                }
                _ => None,
            },
            empty_room_ttl: match properties
                .remove(&PhotonDataType::Byte(game_property_key::EMPTY_ROOM_TTL))
            {
                Some(PhotonDataType::Integer(b)) => Some(b),
                Some(_) => {
                    warn!("Unexpected data type");
                    None
                }
                _ => None,
            },
            player_ttl: match properties
                .remove(&PhotonDataType::Byte(game_property_key::PLAYER_TTL))
            {
                Some(PhotonDataType::Integer(b)) => Some(b),
                Some(_) => {
                    warn!("Unexpected data type");
                    None
                }
                _ => None,
            },
            custom_properties: properties
                .drain(..)
                .filter_map(|(k, v)| match k {
                    PhotonDataType::String(k) => Some((k, v)),
                    _ => {
                        warn!("Unexpected data type");
                        None
                    }
                })
                .collect::<IndexMap<String, PhotonDataType>>(),
        }
    }

    fn into_hashtable(mut self, map: &mut IndexMap<PhotonDataType, PhotonDataType>) {
        if let Some(b) = self.removed.take() {
            map.insert(
                PhotonDataType::Byte(game_property_key::REMOVED),
                PhotonDataType::Boolean(b),
            );
        }
        if let Some(b) = self.max_players.take() {
            map.insert(
                PhotonDataType::Byte(game_property_key::MAX_PLAYERS),
                PhotonDataType::Byte(b),
            );
        }
        if let Some(b) = self.is_open.take() {
            map.insert(
                PhotonDataType::Byte(game_property_key::IS_OPEN),
                PhotonDataType::Boolean(b),
            );
        }
        if let Some(b) = self.is_visible.take() {
            map.insert(
                PhotonDataType::Byte(game_property_key::IS_VISIBLE),
                PhotonDataType::Boolean(b),
            );
        }
        if let Some(b) = self.player_count.take() {
            map.insert(
                PhotonDataType::Byte(game_property_key::PLAYER_COUNT),
                PhotonDataType::Byte(b),
            );
        }
        if let Some(b) = self.cleanup_cache_on_leave.take() {
            map.insert(
                PhotonDataType::Byte(game_property_key::CLEANUP_CACHE_ON_LEAVE),
                PhotonDataType::Boolean(b),
            );
        }
        if let Some(b) = self.master_client_id.take() {
            map.insert(
                PhotonDataType::Byte(game_property_key::MASTER_CLIENT_ID),
                PhotonDataType::Integer(b),
            );
        }
        if let Some(b) = self.props_listed_in_lobby.take() {
            map.insert(
                PhotonDataType::Byte(game_property_key::PROPS_LISTED_IN_LOBBY),
                PhotonDataType::StringArray(b),
            );
        }
        if let Some(b) = self.expected_users.take() {
            map.insert(
                PhotonDataType::Byte(game_property_key::EXPECTED_USERS),
                PhotonDataType::StringArray(b),
            );
        }
        if let Some(b) = self.empty_room_ttl.take() {
            map.insert(
                PhotonDataType::Byte(game_property_key::EMPTY_ROOM_TTL),
                PhotonDataType::Integer(b),
            );
        }
        if let Some(b) = self.player_ttl.take() {
            map.insert(
                PhotonDataType::Byte(game_property_key::PLAYER_TTL),
                PhotonDataType::Integer(b),
            );
        }

        for (k, v) in self.custom_properties.drain(..) {
            map.insert(PhotonDataType::String(k), v);
        }
    }
}
