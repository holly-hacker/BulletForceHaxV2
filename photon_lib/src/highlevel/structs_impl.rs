use super::structs::*;
use super::PhotonMapConversion;

impl RpcEvent {
    /// Drains the [Self::data] field
    pub fn extract_rpc_data(&mut self) -> Option<RpcEventData> {
        self.data.as_mut().map(RpcEventData::from_map)
    }
}

impl RpcEventData {
    const PHOTON_NETWORK_MAX_VIEW_IDS: i32 = 1000;

    pub fn get_owner_id(&self) -> Option<i32> {
        self.net_view_id
            .map(|i| i / Self::PHOTON_NETWORK_MAX_VIEW_IDS)
    }
}
