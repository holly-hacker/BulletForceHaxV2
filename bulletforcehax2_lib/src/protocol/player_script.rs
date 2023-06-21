use photon_lib::{
    photon_data_type::{CustomData, PhotonDataType},
    primitives::{Quaternion, Vector3},
};

#[derive(Debug)]
pub struct PlayerScript {
    /// Value between 0 and 3600 where 0 is straight ahead.
    pub pitch: i16,
    /// Value between 0 and 3600.
    pub yaw: i16,
    /// Related to [Self::yaw].
    pub move_angle: i16, // move angle?
    pub number_of_kills: i16,
    pub number_of_deaths: i16,
    pub number_of_rounds: i16,
    pub ping: i16,
    pub last_local_hit_y: i16, // possibly lastLocalHitY
    pub gun_game_score: i16,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,

    /// Health value, where 10000 is 100%
    pub health: i16,
    pub accessory_type: u8,
    pub barrel_type: u8,
    pub sight_type: u8,

    /// The weapon of the person that last damaged this player.
    pub weapon_last_damaged_from: u8, // damager weapon

    pub bitflags: u8, // bitflags: Crouching, CanShoot, GunReloading, IsThrowing, IsGrounded, 0, 0, 0

    /// The id of the person that last damaged this player.
    pub last_damager_id: i32,

    pub position: Vector3,
    pub rotation: Quaternion,
}

impl PlayerScript {
    pub fn from_object_array(objects: &[PhotonDataType]) -> anyhow::Result<Self> {
        Ok(Self {
            pitch: match objects.get(0) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 0"),
            },
            yaw: match objects.get(1) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 1"),
            },
            move_angle: match objects.get(2) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 2"),
            },
            number_of_kills: match objects.get(3) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 3"),
            },
            number_of_deaths: match objects.get(4) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 4"),
            },
            number_of_rounds: match objects.get(5) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 5"),
            },
            ping: match objects.get(6) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 6"),
            },
            last_local_hit_y: match objects.get(7) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 7"),
            },
            gun_game_score: match objects.get(8) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 8"),
            },
            velocity_x: match objects.get(9) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 9"),
            },
            velocity_y: match objects.get(10) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 10"),
            },
            velocity_z: match objects.get(11) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 11"),
            },
            health: match objects.get(12) {
                Some(PhotonDataType::Short(x)) => *x,
                _ => anyhow::bail!("Expected type Short in PlayerScript position 12"),
            },
            accessory_type: match objects.get(13) {
                Some(PhotonDataType::Byte(x)) => *x,
                _ => anyhow::bail!("Expected type Byte in PlayerScript position 13"),
            },
            barrel_type: match objects.get(14) {
                Some(PhotonDataType::Byte(x)) => *x,
                _ => anyhow::bail!("Expected type Byte in PlayerScript position 14"),
            },
            sight_type: match objects.get(15) {
                Some(PhotonDataType::Byte(x)) => *x,
                _ => anyhow::bail!("Expected type Byte in PlayerScript position 15"),
            },
            weapon_last_damaged_from: match objects.get(16) {
                Some(PhotonDataType::Byte(x)) => *x,
                _ => anyhow::bail!("Expected type Byte in PlayerScript position 16"),
            },
            bitflags: match objects.get(17) {
                Some(PhotonDataType::Byte(x)) => *x,
                _ => anyhow::bail!("Expected type Byte in PlayerScript position 17"),
            },
            last_damager_id: match objects.get(18) {
                Some(PhotonDataType::Integer(x)) => *x,
                _ => anyhow::bail!("Expected type Integer in PlayerScript position 18"),
            },
            position: match objects.get(19) {
                Some(PhotonDataType::Custom(CustomData::Vector3(x))) => *x,
                _ => anyhow::bail!("Expected type Vector3 in PlayerScript position 19"),
            },
            rotation: match objects.get(20) {
                Some(PhotonDataType::Custom(CustomData::Quaternion(x))) => *x,
                _ => anyhow::bail!("Expected type Quaternion in PlayerScript position 20"),
            },
        })
    }
}
