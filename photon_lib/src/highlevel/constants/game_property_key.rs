/// <summary>(255) Max number of players that "fit" into this room. 0 is for "unlimited".</summary>
pub const MAX_PLAYERS: u8 = 255;

/// <summary>(254) Makes this room listed or not in the lobby on master.</summary>
pub const IS_VISIBLE: u8 = 254;

/// <summary>(253) Allows more players to join a room (or not).</summary>
pub const IS_OPEN: u8 = 253;

/// <summary>(252) Current count of players in the room. Used only in the lobby on master.</summary>
pub const PLAYER_COUNT: u8 = 252;

/// <summary>(251) True if the room is to be removed from room listing (used in update to room list in lobby on master)</summary>
pub const REMOVED: u8 = 251;

/// <summary>(250) A list of the room properties to pass to the RoomInfo list in a lobby. This is used in CreateRoom, which defines this list once per room.</summary>
pub const PROPS_LISTED_IN_LOBBY: u8 = 250;

/// <summary>(249) Equivalent of Operation Join parameter CleanupCacheOnLeave.</summary>
pub const CLEANUP_CACHE_ON_LEAVE: u8 = 249;

/// <summary>(248) Code for MasterClientId, which is synced by server. When sent as op-parameter this is (byte)203. As room property this is (byte)248.</summary>
/// <remarks>Tightly related to ParameterCode.MasterClientId.</remarks>
pub const MASTER_CLIENT_ID: u8 = 248;

/// <summary>(247) Code for ExpectedUsers in a room. Matchmaking keeps a slot open for the players with these userIDs.</summary>
pub const EXPECTED_USERS: u8 = 247;

/// <summary>(246) Player Time To Live. How long any player can be inactive (due to disconnect or leave) before the user gets removed from the playerlist (freeing a slot).</summary>
pub const PLAYER_TTL: u8 = 246;

/// <summary>(245) Room Time To Live. How long a room stays available (and in server-memory), after the last player becomes inactive. After this time, the room gets persisted or destroyed.</summary>
pub const EMPTY_ROOM_TTL: u8 = 245;
