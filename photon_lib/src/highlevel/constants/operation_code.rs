/// ObsoleteO Exchanging encrpytion keys is done internally in the lib now. Don't expect this operation-result.
pub const EXCHANGE_KEYS_FOR_ENCRYPTION: u8 = 250;

/// (255) Code for OpJoin, to get into a room.
/// Obsolete
pub const JOIN: u8 = 255;

/// (231) Authenticates this peer and connects to a virtual application
pub const AUTHENTICATE_ONCE: u8 = 231;

/// (230) Authenticates this peer and connects to a virtual application
pub const AUTHENTICATE: u8 = 230;

/// (229) Joins lobby (on master)
pub const JOIN_LOBBY: u8 = 229;

/// (228) Leaves lobby (on master)
pub const LEAVE_LOBBY: u8 = 228;

/// (227) Creates a game (or fails if name exists)
pub const CREATE_GAME: u8 = 227;

/// (226) Join game (by name)
pub const JOIN_GAME: u8 = 226;

/// (225) Joins random game (on master)
pub const JOIN_RANDOM_GAME: u8 = 225;

/// obsolete, cause JoinRandom no longer is a "process". now provides result immediately
pub const CANCEL_JOIN_RANDOM: u8 = 224;

/// (254) Code for OpLeave, to get out of a room.
pub const LEAVE: u8 = 254;

/// (253) Raise event (in a room, for other actors/players)
pub const RAISE_EVENT: u8 = 253;

/// (252) Set Properties (of room or actor/player)
pub const SET_PROPERTIES: u8 = 252;

/// (251) Get Properties
pub const GET_PROPERTIES: u8 = 251;

/// (248) Operation code to change interest groups in Rooms (Lite application and extending ones).
pub const CHANGE_GROUPS: u8 = 248;

/// (222) Request the rooms and online status for a list of friends (by name, which should be unique).
pub const FIND_FRIENDS: u8 = 222;

/// (221) Request statistics about a specific list of lobbies (their user and game count).
pub const GET_LOBBY_STATS: u8 = 221;

/// (220) Get list of regional servers from a NameServer.
pub const GET_REGIONS: u8 = 220;

/// (219) WebRpc Operation.
pub const WEB_RPC: u8 = 219;

/// (218) Operation to set some server settings. Used with different parameters on various servers.
pub const SERVER_SETTINGS: u8 = 218;

/// (217) Get the game list matching a supplied sql filter (SqlListLobby only)
pub const GET_GAME_LIST: u8 = 217;
