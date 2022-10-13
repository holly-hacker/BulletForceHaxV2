//! Keys used by [RoomInfo]
//!
//! Note that the documentation and deprecation attributes come from Photon with only minor edits.

#[allow(unused)]
use crate::highlevel::{constants::*, lobby::RoomInfo};

/// (255) Max number of players that "fit" into this room. 0 is for "unlimited".
pub const MAX_PLAYERS: u8 = 255;

/// (254) Makes this room listed or not in the lobby on master.
pub const IS_VISIBLE: u8 = 254;

/// (253) Allows more players to join a room (or not).
pub const IS_OPEN: u8 = 253;

/// (252) Current count of players in the room. Used only in the lobby on master.
pub const PLAYER_COUNT: u8 = 252;

/// (251) True if the room is to be removed from room listing (used in update to room list in lobby on master)
pub const REMOVED: u8 = 251;

/// (250) A list of the room properties to pass to the RoomInfo list in a lobby. This is used in CreateRoom, which defines this list once per room.
pub const PROPS_LISTED_IN_LOBBY: u8 = 250;

/// (249) Equivalent of [operation_code::JOIN] parameter [parameter_code::CLEANUP_CACHE_ON_LEAVE].
pub const CLEANUP_CACHE_ON_LEAVE: u8 = 249;

/// (248) Code for MasterClientId, which is synced by server. When sent as op-parameter this is (byte)203. As room property this is (byte)248.
///
/// # Remarks
/// Tightly related to [parameter_code::MASTER_CLIENT_ID].
pub const MASTER_CLIENT_ID: u8 = 248;

/// (247) Code for ExpectedUsers in a room. Matchmaking keeps a slot open for the players with these userIDs.
pub const EXPECTED_USERS: u8 = 247;

/// (246) Player Time To Live. How long any player can be inactive (due to disconnect or leave) before the user gets removed from the playerlist (freeing a slot).
pub const PLAYER_TTL: u8 = 246;

/// (245) Room Time To Live. How long a room stays available (and in server-memory), after the last player becomes inactive. After this time, the room gets persisted or destroyed.
pub const EMPTY_ROOM_TTL: u8 = 245;
