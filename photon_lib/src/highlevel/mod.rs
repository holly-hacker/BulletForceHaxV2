//! Implements high-level types PhotonRealtime, PhotonUnityNetworking and PhotonChat.

#[macro_use]
mod macro_impl;

pub mod constants;
pub mod gameplay;
pub mod lobby;

use indexmap::IndexMap;

use crate::photon_data_type::PhotonDataType;

/// Allows conversion between a type and a photon hashmap with byte keys.
pub trait PhotonParameterMapConversion {
    fn from_map(properties: &mut IndexMap<u8, PhotonDataType>) -> Self;
    fn into_map(self, map: &mut IndexMap<u8, PhotonDataType>);
}

/// Allows conversion between a type and a photon hashmap.
pub trait PhotonMapConversion {
    fn from_map(properties: &mut IndexMap<PhotonDataType, PhotonDataType>) -> Self;
    fn into_map(self, map: &mut IndexMap<PhotonDataType, PhotonDataType>);
}
