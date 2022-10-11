//! Implements high-level types PhotonRealtime, PhotonUnityNetworking and PhotonChat.

#[macro_use]
mod macro_impl;

pub mod constants;
pub mod gameplay;
pub mod lobby;

use indexmap::IndexMap;

use crate::photon_data_type::PhotonDataType;

pub trait PhotonParameterMapConversion {
    fn from_map(properties: &mut IndexMap<u8, PhotonDataType>) -> Self;
    fn into_map(self, map: &mut IndexMap<u8, PhotonDataType>);
}

pub trait PhotonMapConversion {
    fn from_map(properties: &mut IndexMap<PhotonDataType, PhotonDataType>) -> Self;
    fn into_map(self, map: &mut IndexMap<PhotonDataType, PhotonDataType>);
}
