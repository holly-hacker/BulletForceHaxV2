macro_rules! impl_photon_map_conversion {
    (
        $(#[$attr:meta])*
        $type_name:ident {
            $(
                #[$photon_key:expr => $photon_value:path]
                $field_name:ident: $field_type:ty,
            )+
        }
    ) => {
        $(#[$attr])*
        pub struct $type_name {
            $(
                pub $field_name: Option<$field_type>,
            )+

            pub custom_properties: indexmap::IndexMap<String, PhotonDataType>,
        }

        impl PhotonMapConversion for $type_name {
            fn from_map(properties: &mut indexmap::IndexMap<PhotonDataType, PhotonDataType>) -> Self {
                $type_name {
                    $(
                        // NOTE: we need to use `shift_remove` to retain order for custom_properties later
                        // this may not actually be important, but it allows types converted both ways and be identical
                        $field_name: match properties.shift_remove(&$photon_key) {
                            Some($photon_value(b)) => Some(b),
                            Some(_) => {
                                tracing::warn!("Unexpected data type");
                                None
                            }
                            _ => None,
                        },
                    )+

                    custom_properties: properties
                        .drain(..)
                        .filter_map(|(k, v)| match k {
                            PhotonDataType::String(k) => Some((k, v)),
                            _ => {
                                tracing::warn!("Unexpected data type");
                                None
                            }
                        })
                        .collect::<indexmap::IndexMap<String, PhotonDataType>>(),
                }
            }

            fn into_map(mut self, map: &mut indexmap::IndexMap<PhotonDataType, PhotonDataType>) {
                $(
                    if let Some(b) = self.$field_name.take() {
                        map.insert($photon_key, $photon_value(b));
                    }
                )+

                for (k, v) in self.custom_properties.drain(..) {
                    map.insert(PhotonDataType::String(k), v);
                }
            }
        }
    };
}
