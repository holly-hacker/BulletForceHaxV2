use super::structs::*;
use super::PhotonMapConversion;

impl RpcEvent {
    /// Drains the [Self::data] field
    pub fn extract_rpc_call(&mut self) -> Option<RpcCall> {
        self.data.as_mut().map(RpcCall::from_map)
    }
}

impl RpcCall {
    const PHOTON_NETWORK_MAX_VIEW_IDS: i32 = 1000;

    pub fn get_owner_id(&self) -> i32 {
        self.net_view_id / Self::PHOTON_NETWORK_MAX_VIEW_IDS
    }
}
