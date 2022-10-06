macro_rules! impl_hashtable {
    ($type_name:ident {
        $(
            #[$photon_key:expr => $photon_value:path]
            $field_name:ident: $field_type:ty,
        )+
    }) => {
        #[derive(Debug)]
        pub struct $type_name {
            $(
                pub $field_name: Option<$field_type>,
            )+

            pub custom_properties: indexmap::IndexMap<String, PhotonDataType>,
        }

        impl EventDataBased for $type_name {
            fn from_hashtable(properties: &mut indexmap::IndexMap<PhotonDataType, PhotonDataType>) -> Self {
                $type_name {
                    $(
                        $field_name: match properties.remove(&$photon_key) {
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

            fn into_hashtable(mut self, map: &mut indexmap::IndexMap<PhotonDataType, PhotonDataType>) {
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
