//! High-level representations of photon messages.

pub use super::structs_impl::*;
use crate::highlevel::constants::{actor_properties, game_property_key, parameter_code};
#[allow(unused)]
use crate::highlevel::constants::{event_code, operation_code, pun_event_code};
use crate::photon_data_type::{CustomData, PhotonDataType};
use crate::PhotonHashmap;

/// Represents a Photon View ID
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewId(pub i32);

// NOTE: be very cautious when applying `@required`, parsing will fail if it is not present!
// if you cannot prove that a property is actually always present, do not apply it.
// Basically, always be more cautious than PUN is.

impl_u8_map_conversion! {
    /// Parameter of [event_code::GAME_LIST] and [event_code::GAME_LIST_UPDATE]. Contains a list of [RoomInfo].
    RoomInfoList {
        @required
        [parameter_code::GAME_LIST => PhotonDataType::Hashtable]
        games: PhotonHashmap,
    }

    SetPropertiesOperationRequest {
        @required
        [parameter_code::PROPERTIES => PhotonDataType::Hashtable]
        properties: PhotonHashmap,

        /// Only present when updating an actor, not when updating a room.
        [parameter_code::ACTOR_NR => PhotonDataType::Integer]
        actor_nr: i32,

        @required
        [parameter_code::BROADCAST => PhotonDataType::Boolean]
        broadcast: bool,

        [parameter_code::EXPECTED_VALUES => PhotonDataType::Hashtable]
        expected_values: PhotonHashmap,

        [parameter_code::EVENT_FORWARD => PhotonDataType::Boolean]
        event_forward: bool,
    }

    /// Request parameter of [operation_code::JOIN_GAME].
    #[derive(Debug)]
    JoinGameRequest {
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

    /// Response parameter of [operation_code::JOIN_GAME] on success (return code 0).
    #[derive(Debug)]
    JoinGameResponseSuccess {
        [parameter_code::ROOM_NAME => PhotonDataType::String]
        room_name: String,

        @required
        [parameter_code::ACTOR_NR => PhotonDataType::Integer]
        actor_nr: i32,

        [parameter_code::ACTOR_LIST => PhotonDataType::IntArray]
        actor_list: Vec<i32>,

        /// A hashmap over serialized [Player]s. The keys in this hashmap are integer actor ids.
        @required
        [parameter_code::PLAYER_PROPERTIES => PhotonDataType::Hashtable]
        player_properties: PhotonHashmap,

        /// A serialized instance of [RoomInfo]
        @required
        [parameter_code::GAME_PROPERTIES => PhotonDataType::Hashtable]
        game_properties: PhotonHashmap,

        [parameter_code::ADDRESS => PhotonDataType::String]
        address: String,

        [parameter_code::ROOM_OPTION_FLAGS => PhotonDataType::Integer]
        room_option_flags: i32, // could add an impl to map this to an enum or something
    }

    /// Request parameter of [operation_code::RAISE_EVENT]
    #[derive(Debug)]
    RaiseEvent {
        @required
        [parameter_code::CODE => PhotonDataType::Byte]
        event_code: u8,

        [parameter_code::DATA]
        data: PhotonDataType,

        [parameter_code::CACHE => PhotonDataType::Byte]
        cache: u8,

        [parameter_code::RECEIVER_GROUP => PhotonDataType::Byte]
        receiver_group: u8,

        [parameter_code::GROUP => PhotonDataType::Byte]
        interest_group: u8,

        [parameter_code::ACTOR_LIST => PhotonDataType::IntArray]
        actor_list: Vec<i32>,

        [parameter_code::EVENT_FORWARD => PhotonDataType::Boolean]
        event_forward: bool,
    }

    /// Parameter for [event_code::LEAVE].
    #[derive(Debug)]
    LeaveEvent {
        /// The id of the player who left
        [parameter_code::ACTOR_NR => PhotonDataType::Integer]
        sender_actor: i32,

        [parameter_code::ACTOR_LIST => PhotonDataType::Array]
        actors: Vec<PhotonDataType>,

        [parameter_code::IS_INACTIVE => PhotonDataType::Boolean]
        is_inactive: bool,

        /// The new master client
        [parameter_code::MASTER_CLIENT_ID => PhotonDataType::Integer]
        master_client_id: i32,
    }

    /// Parameter for [event_code::PROPERTIES_CHANGED].
    #[derive(Debug)]
    PropertiesChangedEvent {
        /// The id of the player who left
        [parameter_code::ACTOR_NR => PhotonDataType::Integer]
        sender_actor: i32,

        @required
        [parameter_code::TARGET_ACTOR_NR => PhotonDataType::Integer]
        target_actor_number: i32,

        /// If [Self::target_actor_number] is 0, these are game properties. Otherwise these are actor properties.
        @required
        [parameter_code::PROPERTIES => PhotonDataType::Hashtable]
        properties: PhotonHashmap,
    }

    /// Parameter for [pun_event_code::DESTROY].
    DestroyEvent {
        [parameter_code::ACTOR_NR => PhotonDataType::Integer]
        sender_actor: i32,

        @required
        [parameter_code::DATA => PhotonDataType::Hashtable]
        data: PhotonHashmap,
    }

    /// Parameter for [pun_event_code::INSTANTIATION].
    InstantiationEvent {
        [parameter_code::ACTOR_NR => PhotonDataType::Integer]
        sender_actor: i32,

        @required
        [parameter_code::DATA => PhotonDataType::Hashtable]
        data: PhotonHashmap,
    }

    /// Parameter for [pun_event_code::SEND_SERIALIZE] and [pun_event_code::SEND_SERIALIZE_RELIABLE].
    SendSerializeEvent {
        [parameter_code::ACTOR_NR => PhotonDataType::Integer]
        sender_actor: i32,

        @required
        [parameter_code::DATA => PhotonDataType::Hashtable]
        data: PhotonHashmap,
    }

    /// Parameter for [pun_event_code::RPC]. Contains a single [RpcCall].
    #[derive(Debug)]
    RpcEvent {
        [parameter_code::ACTOR_NR => PhotonDataType::Integer]
        sender_actor: i32,

        @required
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
    #[derive(Debug)]
    Player {
        // It's possible that this is always present, but I'm not 100% sure.
        /// The player's nickname.
        [PhotonDataType::Byte(actor_properties::PLAYER_NAME) => PhotonDataType::String]
        nickname: String,

        // Surprisingly, not always present.
        [PhotonDataType::Byte(actor_properties::USER_ID) => PhotonDataType::String]
        user_id: String,

        [PhotonDataType::Byte(actor_properties::IS_INACTIVE) => PhotonDataType::Boolean]
        is_inactive: bool,
    }

    /// Event data from [DestroyEvent].
    #[derive(Debug)]
    DestroyEventData {
        @required
        [PhotonDataType::Byte(0) => PhotonDataType::Integer]
        view_id: i32,
    }

    /// Event data from [InstantiationEvent].
    #[derive(Debug)]
    InstantiationEventData {
        @required
        [PhotonDataType::Byte(0) => PhotonDataType::String]
        prefab_name: String,

        /// # Remarks
        /// Of type Vector3
        [PhotonDataType::Byte(1) => PhotonDataType::Custom]
        position: CustomData,

        /// # Remarks
        /// Of type Quaternion
        [PhotonDataType::Byte(2) => PhotonDataType::Custom]
        rotation: CustomData,

        [PhotonDataType::Byte(3) => PhotonDataType::Byte]
        group: u8,

        /// Should be of same length as [Self::incoming_instantiation_data].
        [PhotonDataType::Byte(4) => PhotonDataType::IntArray]
        views_ids: Vec<i32>,

        /// Should be of same length as [Self::views_ids].
        [PhotonDataType::Byte(5) => PhotonDataType::ObjectArray]
        incoming_instantiation_data: Vec<PhotonDataType>,

        @required
        [PhotonDataType::Byte(6) => PhotonDataType::Integer]
        server_time: i32,

        /// The view id
        @required
        [PhotonDataType::Byte(7) => PhotonDataType::Integer]
        instantiation_id: i32,

        [PhotonDataType::Byte(8) => PhotonDataType::Short]
        obj_level_prefix: i16,
    }

    /// An RPC call. Can be both sent and received by the client.
    RpcCall {
        @required
        [PhotonDataType::Byte(0) => PhotonDataType::Integer]
        net_view_id: i32,

        [PhotonDataType::Byte(1) => PhotonDataType::Short]
        other_side_prefix: i16,

        /// Present when sent from client to server, but not the other way around
        [PhotonDataType::Byte(2) => PhotonDataType::Integer]
        server_timestamp: i32,

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

/// A serialized object stream. Can represent a `Monobehavior`, a `Transform`, a `Rigidbody` or a `RigidBody2D`.
///
/// See [SendSerializeEvent].
pub struct SerializedData {
    pub view_id: i32,
    pub data_stream: Vec<PhotonDataType>,
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
            let serialized = RoomInfo::from_map(&mut photon_map.clone()).unwrap();
            assert_eq!(serialized, room_info);
        }
    }
}
