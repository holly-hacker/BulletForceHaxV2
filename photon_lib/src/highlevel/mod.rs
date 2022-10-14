//! Implements high-level types PhotonRealtime, PhotonUnityNetworking and PhotonChat.

#[macro_use]
mod macro_impl;

pub mod constants;
pub mod gameplay;
pub mod lobby;

use crate::{ParameterMap, PhotonHashmap};

/// Allows conversion between a type and a photon hashmap with byte keys.
pub trait PhotonParameterMapConversion {
    fn from_map(properties: &mut ParameterMap) -> Self;
    fn into_map(self, map: &mut ParameterMap);
}

/// Allows conversion between a type and a photon hashmap.
pub trait PhotonMapConversion {
    fn from_map(properties: &mut PhotonHashmap) -> Self;
    fn into_map(self, map: &mut PhotonHashmap);
}
