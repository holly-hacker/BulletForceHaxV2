//! Describes keys for the parameter map in types such as [EventData], [OperationRequest] and [OperationResponse].
//!
//! Note that the documentation and deprecation attributes come from Photon with only minor edits.

#[allow(unused)]
use crate::{
    highlevel::constants::*,
    photon_message::{EventData, OperationRequest, OperationResponse},
};

/// (255) Code for the gameId/roomName (a unique name per room). Used in [operation_code::JOIN] and similar.
pub const ROOM_NAME: u8 = 255;

/// (254) Code of the Actor of an operation. Used for property get and set.
pub const ACTOR_NR: u8 = 254;

/// (253) Code of the target Actor of an operation. Used for property set. Is 0 for game
pub const TARGET_ACTOR_NR: u8 = 253;

/// (252) Code for list of players in a room.
pub const ACTOR_LIST: u8 = 252;

/// (251) Code for property-set (Hashtable). This key is used when sending only one set of properties.
/// If either ActorProperties or GameProperties are used (or both), check those keys.
pub const PROPERTIES: u8 = 251;

/// (250) Code for broadcast parameter of [operation_code::SET_PROPERTIES] method.
pub const BROADCAST: u8 = 250;

/// (249) Code for property set (Hashtable).
pub const PLAYER_PROPERTIES: u8 = 249;

/// (248) Code for property set (Hashtable).
pub const GAME_PROPERTIES: u8 = 248;

/// (247) Code for caching events while raising them.
pub const CACHE: u8 = 247;

/// (246) Code to select the receivers of events (used in Lite, Operation [operation_code::RAISE_EVENT]).
pub const RECEIVER_GROUP: u8 = 246;

/// (245) Code of data/custom content of an event. Used in [operation_code::RAISE_EVENT].
pub const CUSTOM_EVENT_CONTENT: u8 = 245;

/// (245) Code of data of an event. Used in [operation_code::RAISE_EVENT].
pub const DATA: u8 = 245;

/// (244) Code used when sending some code-related parameter, like [operation_code::RAISE_EVENT]'s event-code.
///
/// # Remarks
/// This is not the same as the Operation's code, which is no longer sent as part of the parameter Dictionary in Photon 3.
pub const CODE: u8 = 244;

/// (241) Bool parameter of [operation_code::CREATE_GAME] Operation. If true, server cleans up roomcache of leaving players (their cached events get removed).
pub const CLEANUP_CACHE_ON_LEAVE: u8 = 241;

/// (240) Code for "group" operation-parameter (as used in [operation_code::RAISE_EVENT]).
pub const GROUP: u8 = 240;

/// (239) The "Remove" operation-parameter can be used to remove something from a list. E.g. remove groups from player's interest groups.
pub const REMOVE: u8 = 239;

/// (239) Used in [operation_code::JOIN] to define if UserIds of the players are broadcast in the room. Useful for FindFriends and reserving slots for expected users.
pub const PUBLISH_USER_ID: u8 = 239;

/// (238) The "Add" operation-parameter can be used to add something to some list or set. E.g. add groups to player's interest groups.
pub const ADD: u8 = 238;

/// (237) A bool parameter for creating games. If set to true, no room events are sent to the clients on join and leave. Default: false (and not sent).
pub const SUPPRESS_ROOM_EVENTS: u8 = 237;

/// (236) Time To Live (TTL) for a room when the last player leaves. Keeps room in memory for case a player re-joins soon. In milliseconds.
pub const EMPTY_ROOM_TTL: u8 = 236;

/// (235) Time To Live (TTL) for an 'actor' in a room. If a client disconnects, this actor is inactive first and removed after this timeout. In milliseconds.
pub const PLAYER_TTL: u8 = 235;

/// (234) Optional parameter of [operation_code::RAISE_EVENT] and OpSetCustomProperties to forward the event/operation to a web-service.
pub const EVENT_FORWARD: u8 = 234;

/// (233) Optional parameter of [operation_code::LEAVE] in async games. If false, the player does abandons the game (forever). By default players become inactive and can re-join.
#[deprecated(note = "Use: IsInactive")]
pub const IS_COMING_BACK: u8 = 233;

/// (233) Used in EvLeave to describe if a user is inactive (and might come back) or not. In rooms with PlayerTTL, becoming inactive is the default case.
pub const IS_INACTIVE: u8 = 233;

/// (232) Used when creating rooms to define if any userid can join the room only once.
pub const CHECK_USER_ON_JOIN: u8 = 232;

/// (231) Code for "Check And Swap" (CAS) when changing properties.
pub const EXPECTED_VALUES: u8 = 231;

/// (230) Address of a (game) server to use.
pub const ADDRESS: u8 = 230;

/// (229) Count of players in this application in a rooms (used in stats event)
pub const PEER_COUNT: u8 = 229;

/// (228) Count of games in this application (used in stats event)
pub const GAME_COUNT: u8 = 228;

/// (227) Count of players on the master server (in this app, looking for rooms)
pub const MASTER_PEER_COUNT: u8 = 227;

/// (225) User's ID
pub const USER_ID: u8 = 225;

/// (224) Your application's ID: a name on your own Photon or a GUID on the Photon Cloud
pub const APPLICATION_ID: u8 = 224;

/// (223) Not used currently (as "Position"). If you get queued before connect, this is your position
pub const POSITION: u8 = 223;

/// (223) Modifies the matchmaking algorithm used for [operation_code::JOIN_RANDOM_GAME]. Allowed parameter values are defined in enum MatchmakingMode.
pub const MATCH_MAKING_TYPE: u8 = 223;

/// (222) List of RoomInfos about open / listed rooms
pub const GAME_LIST: u8 = 222;

/// (221) Internally used to establish encryption
pub const TOKEN: u8 = 221;

/// (220) Version of your application
pub const APP_VERSION: u8 = 220;

/// (218) Content for [event_code::ERROR_INFO] and internal debug operations.
pub const INFO: u8 = 218;

/// (217) This key's (byte) value defines the target custom authentication type/service the client connects with. Used in [operation_code::AUTHENTICATE]
pub const CLIENT_AUTHENTICATION_TYPE: u8 = 217;

/// (216) This key's (string) value provides parameters sent to the custom authentication type/service the client connects with. Used in [operation_code::AUTHENTICATE]
pub const CLIENT_AUTHENTICATION_PARAMS: u8 = 216;

/// (215) Makes the server create a room if it doesn't exist. [operation_code::JOIN] uses this to always enter a room, unless it exists and is full/closed.
pub const CREATE_IF_NOT_EXISTS: u8 = 215;

/// (215) The JoinMode enum defines which variant of joining a room will be executed: Join only if available, create if not exists or re-join.
///
/// # Remarks
/// Replaces [CREATE_IF_NOT_EXISTS] which was only a bool-value.
pub const JOIN_MODE: u8 = 215;

/// (214) This key's (string or byte[]) value provides parameters sent to the custom authentication service setup in Photon Dashboard. Used in [operation_code::AUTHENTICATE]
pub const CLIENT_AUTHENTICATION_DATA: u8 = 214;

/// (213) Used in matchmaking-related methods and when creating a room to name a lobby (to join or to attach a room to).
pub const LOBBY_NAME: u8 = 213;

/// (212) Used in matchmaking-related methods and when creating a room to define the type of a lobby. Combined with the lobby name this identifies the lobby.
pub const LOBBY_TYPE: u8 = 212;

/// (211) This (optional) parameter can be sent in [operation_code::AUTHENTICATE] to turn on Lobby Stats (info about lobby names and their user- and game-counts).
pub const LOBBY_STATS: u8 = 211;

/// (210) Used for region values in [operation_code::AUTHENTICATE] and [operation_code::GET_REGIONS].
pub const REGION: u8 = 210;

/// (210) Internally used in case of hosting by Azure
/// only used within events, so use: [event_code::AZURE_NODE_INFO]
#[deprecated(note = "TCP routing was removed after becoming obsolete.")]
pub const AZURE_NODE_INFO: u8 = 210;

/// (209) Path of the WebRPC that got called. Also known as "WebRpc Name". Type: string.
pub const URI_PATH: u8 = 209;

/// (209) Internally used in case of hosting by Azure
#[deprecated(note = "TCP routing was removed after becoming obsolete.")]
pub const AZURE_LOCAL_NODE_ID: u8 = 209;

/// (208) Internally used in case of hosting by Azure
#[deprecated(note = "TCP routing was removed after becoming obsolete.")]
pub const AZURE_MASTER_NODE_ID: u8 = 208;

/// (208) Parameters for a WebRPC as: Dictionary&lt;string, object&gt;. This will get serialized to JSon.
pub const WEB_RPC_PARAMETERS: u8 = 208;

/// (207) ReturnCode for the WebRPC, as sent by the web service (not by Photon, which uses ErrorCode). Type: byte.
pub const WEB_RPC_RETURN_CODE: u8 = 207;

/// (206) Message returned by WebRPC server. Analog to Photon's debug message. Type: string.
pub const WEB_RPC_RETURN_MESSAGE: u8 = 206;

// NOTE: everything under here is NOT present in the version of Photon that BulletForce uses.

/// (205) Used to define a "slice" for cached events. Slices can easily be removed from cache. Type: int.
pub const CACHE_SLICE_INDEX: u8 = 205;

/// (204) Informs the server of the expected plugin setup.
/// <remarks>
/// The operation will fail in case of a plugin mismatch returning error code PluginMismatch 32751(0x7FFF - 16).
/// Setting string[]{} means the client expects no plugin to be setup.
/// Note: for backwards compatibility null omits any check.
/// </remarks>
pub const PLUGINS: u8 = 204;

/// (203) Code for MasterClientId, which is synced by server. When sent as op-parameter this is code 203.
/// <remarks>Tightly related to GamePropertyKey.MasterClientId.</remarks>
pub const MASTER_CLIENT_ID: u8 = 203;

/// (202) Used by the server in Operation Responses, when it sends the nickname of the client (the user's nickname).
pub const NICK_NAME: u8 = 202;

/// (201) Informs user about name of plugin load to game
pub const PLUGIN_NAME: u8 = 201;

/// (200) Informs user about version of plugin load to game
pub const PLUGIN_VERSION: u8 = 200;

/// (196) Cluster info provided in [operation_code::AUTHENTICATE]/[operation_code::AUTHENTICATE_ONCE] responses.
pub const CLUSTER: u8 = 196;

/// (195) Protocol which will be used by client to connect master/game servers. Used for nameserver.
pub const EXPECTED_PROTOCOL: u8 = 195;

/// (194) Set of custom parameters which are sent in auth request.
pub const CUSTOM_INIT_DATA: u8 = 194;

/// (193) How are we going to encrypt data.
pub const ENCRYPTION_MODE: u8 = 193;

/// (192) Parameter of Authentication, which contains encryption keys (depends on AuthMode and EncryptionMode).
pub const ENCRYPTION_DATA: u8 = 192;

/// (191) An int parameter summarizing several boolean room-options with bit-flags.
pub const ROOM_OPTION_FLAGS: u8 = 191;

/// (2) Used in [operation_code::FIND_FRIENDS] request. An integer containing option-flags to filter the results.
pub const FIND_FRIENDS_OPTIONS: u8 = 2;

/// (2) Used in [operation_code::FIND_FRIENDS] response. Contains string[] of room names ("" where not known or no room joined).
pub const FIND_FRIENDS_RESPONSE_ROOM_ID_LIST: u8 = 2;

/// (1) Used in [operation_code::FIND_FRIENDS] request. Value must be string[] of friends to look up.
pub const FIND_FRIENDS_REQUEST_LIST: u8 = 1;

/// (1) Used in [operation_code::FIND_FRIENDS] response. Contains bool[] list of online states (false if not online).
pub const FIND_FRIENDS_RESPONSE_ONLINE_LIST: u8 = 1;
