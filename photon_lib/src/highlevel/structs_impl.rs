use super::{structs::*, FromMapError, PhotonMapConversion};

impl RpcEvent {
    /// Drains the [Self::data] field
    pub fn extract_rpc_call(&mut self) -> Result<Option<RpcCall>, FromMapError> {
        self.data.as_mut().map(RpcCall::from_map).transpose()
    }
}

impl RpcCall {
    const PHOTON_NETWORK_MAX_VIEW_IDS: i32 = 1000;

    pub fn get_owner_id(&self) -> i32 {
        self.net_view_id / Self::PHOTON_NETWORK_MAX_VIEW_IDS
    }
}
