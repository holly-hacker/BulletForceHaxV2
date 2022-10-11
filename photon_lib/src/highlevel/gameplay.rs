use crate::highlevel::constants::actor_properties;
use crate::photon_data_type::PhotonDataType;

// NOTE: this macro adds a `custom_properties` field for remaining, string-keyed properties
impl_photon_map_conversion! {
    Player {
        #[PhotonDataType::Byte(actor_properties::PLAYER_NAME) => PhotonDataType::String]
        nickname: String,

        #[PhotonDataType::Byte(actor_properties::USER_ID) => PhotonDataType::String]
        user_id: String,

        #[PhotonDataType::Byte(actor_properties::IS_INACTIVE) => PhotonDataType::Boolean]
        is_inactive: bool,
    }
}
