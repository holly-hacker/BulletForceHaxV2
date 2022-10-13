//! Codes describing different internal PUN events for [EventData].
//!
//! Note that the documentation and deprecation attributes come from Photon with only minor edits.

#[allow(unused)]
use crate::photon_message::EventData;

pub const OWNERSHIP_UPDATE: u8 = 212;
pub const VACANT_VIEW_IDS: u8 = 211;
pub const OWNERSHIP_TRANSFER: u8 = 210;
pub const OWNERSHIP_REQUEST: u8 = 209;
/// TS: added to make others remove all GOs of a player
pub const DESTROY_PLAYER: u8 = 207;
/// TS: added this but it's not really needed anymore
pub const SEND_SERIALIZE_RELIABLE: u8 = 206;
pub const REMOVE_CACHED_RP_CS: u8 = 205;
pub const DESTROY: u8 = 204;
pub const CLOSE_CONNECTION: u8 = 203;
pub const INSTANTIATION: u8 = 202;
pub const SEND_SERIALIZE: u8 = 201;
pub const RPC: u8 = 200;
