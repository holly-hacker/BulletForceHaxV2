use crate::{photon_data_type::PhotonDataType, ParameterMap, PhotonHashmap};

pub trait PhotonDataTypeExtensions {
    fn to_parameter_map_lossy(self) -> ParameterMap;
}

impl PhotonDataTypeExtensions for PhotonHashmap {
    fn to_parameter_map_lossy(self) -> ParameterMap {
        let mut ret = ParameterMap::with_capacity(self.len());
        for (k, v) in self.into_iter() {
            if let PhotonDataType::Byte(k) = k {
                ret.insert(k, v);
            }
        }
        ret
    }
}
