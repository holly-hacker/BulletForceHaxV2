//! Keys used by [Player]
//!
//! Note that the documentation and deprecation attributes come from Photon with only minor edits.

#[allow(unused)]
use crate::highlevel::{constants::*, gameplay::Player};

/// (255) Name of a player/actor.
pub const PLAYER_NAME: u8 = 255; // was: 1

/// (254) Tells you if the player is currently in this game (getting events live).
///
/// # Remarks
/// A server-set value for async games, where players can leave the game and return later.
pub const IS_INACTIVE: u8 = 254;

/// (253) UserId of the player. Sent when room gets created with RoomOptions.PublishUserId = true.
pub const USER_ID: u8 = 253;
