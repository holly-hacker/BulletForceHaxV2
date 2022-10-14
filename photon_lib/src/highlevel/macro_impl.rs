// TODO: allow opting out of custom_properties

// dev note: you probably want to read https://danielkeep.github.io/tlborm/book/
// this is a hacky macro, macro_rules is probably not intended for some of the things I'm doing (like req/opt).

macro_rules! impl_u8_map_conversion {
    (
        $(
            $(#[$type_attr:meta])*
            $type_name:ident {
                $(
                    $(#[$field_attr:meta])*
                    [$map_key:expr => $map_type:path]
                    $(@ $field_name_req:ident: $field_type_req:ty,)?
                    $($field_name_opt:ident: $field_type_opt:ty,)?
                )*
            }
        )*
    ) => {
        $(
            $(#[$type_attr])*
            pub struct $type_name {
                $(
                    $(#[$field_attr])*
                    $(pub $field_name_opt: Option<$field_type_opt>,)?
                    $(pub $field_name_req: $field_type_req,)?
                )*
            }

            impl crate::highlevel::PhotonParameterMapConversion for $type_name {
                fn from_map(properties: &mut crate::ParameterMap) -> Self {
                    $type_name {
                        // NOTE: we need to use `shift_remove` to retain order for custom_properties later
                        // this may not actually be important, but it allows types converted both ways and be identical
                        $(
                            $(
                                $field_name_req: match properties.shift_remove(&$map_key) {
                                    Some($map_type(b)) => b,
                                    _ => todo!("error handling in from_map for missing req field"), // TODO: error handling here!!
                                },
                            )?
                            $(
                                $field_name_opt: match properties.shift_remove(&$map_key) {
                                    Some($map_type(b)) => Some(b),
                                    _ => None,
                                },
                            )?
                        )*
                    }
                }

                fn into_map(mut self, map: &mut crate::ParameterMap) {
                    $(
                        $(
                            if let Some(b) = self.$field_name_opt.take() {
                                map.insert($map_key, $map_type(b));
                            }
                        )?
                        $(
                            map.insert($map_key, $map_type(self.$field_name_req));
                        )?
                    )*
                }
            }
        )*
    };
}

macro_rules! impl_photon_map_conversion {
    (
        $(
            $(#[$type_attr:meta])*
            $type_name:ident {
                $(
                    $(#[$field_attr:meta])*
                    [$photon_key:expr => $photon_type:path]
                    $(@ $field_name_req:ident: $field_type_req:ty,)?
                    $($field_name_opt:ident: $field_type_opt:ty,)?
                )*
            }
        )*
    ) => {
        $(
            $(#[$type_attr])*
            pub struct $type_name {
                $(
                    $(#[$field_attr])*
                    $(pub $field_name_opt: Option<$field_type_opt>,)?
                    $(pub $field_name_req: $field_type_req,)?
                )*

                pub custom_properties: indexmap::IndexMap<String, PhotonDataType>,
            }

            impl crate::highlevel::PhotonMapConversion for $type_name {
                fn from_map(properties: &mut crate::PhotonHashmap) -> Self {
                    $type_name {
                        $(
                            // NOTE: we need to use `shift_remove` to retain order for custom_properties later
                            // this may not actually be important, but it allows types converted both ways and be identical
                            $(
                                $field_name_req: match properties.shift_remove(&$photon_key) {
                                    Some($photon_type(b)) => b,
                                    Some(k) => {
                                        tracing::warn!(
                                            "When converting {} from map, found {k:?} when expecting data type {}",
                                            stringify!($type_name), stringify!($photon_type));
                                        todo!("error handling in from_map for wrong type in req field");
                                    }
                                    _ => todo!("error handling in from_map for missing req field"), // TODO: error handling here!!
                                },
                            )?
                            $(
                                $field_name_opt: match properties.shift_remove(&$photon_key) {
                                    Some($photon_type(b)) => Some(b),
                                    Some(k) => {
                                        tracing::warn!(
                                            "When converting {} from map, found {k:?} when expecting data type {}",
                                            stringify!($type_name), stringify!($photon_type));
                                        None
                                    }
                                    _ => None,
                                },
                            )?
                        )*

                        custom_properties: properties
                            .drain(..)
                            .filter_map(|(k, v)| match k {
                                PhotonDataType::String(k) => Some((k, v)),
                                k => {
                                    tracing::warn!(
                                        "When mapping custom props for {} from map, found {k:?} as key when expecting a String",
                                        stringify!($type_name));
                                    None
                                }
                            })
                            .collect::<indexmap::IndexMap<String, PhotonDataType>>(),
                    }
                }

                fn into_map(mut self, map: &mut crate::PhotonHashmap) {
                    $(
                        $(
                            map.insert($photon_key, $photon_type(self.$field_name_req));
                        )?
                        $(
                            if let Some(b) = self.$field_name_opt.take() {
                                map.insert($photon_key, $photon_type(b));
                            }
                        )?
                    )*

                    for (k, v) in self.custom_properties.drain(..) {
                        map.insert(PhotonDataType::String(k), v);
                    }
                }
            }
        )*
    };
}
