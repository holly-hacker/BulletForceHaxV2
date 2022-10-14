//! Implements high-level types PhotonRealtime, PhotonUnityNetworking and PhotonChat.

#[macro_use]
mod macro_impl;

pub mod constants;
pub mod structs;
mod structs_impl;

use thiserror::Error;

use crate::{ParameterMap, PhotonHashmap};

#[derive(Error, Debug)]
#[error("{0}")]
pub struct FromMapError(pub String);

/// Allows conversion between a type and a photon hashmap with byte keys.
pub trait PhotonParameterMapConversion: Sized {
    fn from_map(properties: &mut ParameterMap) -> Result<Self, FromMapError>;
    fn into_map(self, map: &mut ParameterMap);
}

/// Allows conversion between a type and a photon hashmap.
pub trait PhotonMapConversion: Sized {
    fn from_map(properties: &mut PhotonHashmap) -> Result<Self, FromMapError>;
    fn into_map(self, map: &mut PhotonHashmap);
}
