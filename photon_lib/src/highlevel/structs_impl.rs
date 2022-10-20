use super::{structs::*, FromMapError, PhotonMapConversion};
use crate::photon_data_type::PhotonDataType;

const PHOTON_NETWORK_MAX_VIEW_IDS: i32 = 1000;

impl RpcEvent {
    /// Drains the [Self::data] field
    pub fn extract_rpc_call(&mut self) -> Result<RpcCall, FromMapError> {
        RpcCall::from_map(&mut self.data)
    }
}

impl SendSerializeEvent {
    // TODO: proper error type. probably want something generic like InvalidDataError
    /// Gets a copy of the serialized data in this event
    pub fn get_serialized_data(&self) -> Option<Vec<SerializedData>> {
        _ = self.data.get(&PhotonDataType::Byte(1))?;

        let header_len = match self.data.contains_key(&PhotonDataType::Byte(1)) {
            true => 2,
            false => 1,
        };

        const DATA_INITIAL_INDEX: usize = 10;
        let data_len = self.data.len() - header_len;
        let mut ret = vec![];
        for i in 0..data_len {
            // items start at key 10 and count up
            // the official implementation has various opportunities to crash here, but we should be safe
            let index = i + DATA_INITIAL_INDEX;
            let index = (index & 0xFF) as u8; // NOTE: official implementation does wrap here

            let found = self.data.get(&PhotonDataType::Byte(index));
            let found = match found {
                Some(PhotonDataType::ObjectArray(x)) => x,
                _ => return None,
            };
            let data = SerializedData::from_object_array(found.clone());
            let data = match data {
                Some(s) => s,
                None => return None,
            };
            ret.push(data);
        }

        Some(ret)
    }
}

impl SerializedData {
    // TODO: views can have specific synchronisation. User should pass that to this method
    // TODO: better errors
    pub fn from_object_array(mut data: Vec<PhotonDataType>) -> Option<Self> {
        if data.len() < 3 {
            return None;
        }

        let view_id = match data[0] {
            PhotonDataType::Integer(i) => i,
            _ => return None,
        };
        // index 1 and 2 are related to compression, which is not implemented yet
        let data_stream = data.drain(3..).collect();

        Some(Self {
            view_id,
            data_stream,
        })
    }

    pub fn get_owner_id(&self) -> i32 {
        self.view_id / PHOTON_NETWORK_MAX_VIEW_IDS
    }

    // NOTE: could add an implementation to parse this component as a Transform, Rigidbody or Rigidbody2D, as they have
    // pre-defined data streams. The user would have to pass their `onSerializeTransformOption` or
    // `onSerializeRigidBodyOption` value, though.
}

impl RpcCall {
    pub fn get_owner_id(&self) -> i32 {
        self.net_view_id / PHOTON_NETWORK_MAX_VIEW_IDS
    }
}
