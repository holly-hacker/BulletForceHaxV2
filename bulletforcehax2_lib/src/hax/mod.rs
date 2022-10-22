//! The main module of BulletForceHaxV2.

mod hax_impl;
mod impl_proxy;

use std::sync::Arc;

use photon_lib::{
    highlevel::structs::{InstantiationEventData, Player, ViewId},
    indexmap::IndexMap,
    photon_data_type::PhotonDataType,
    primitives::Vector3,
};
use tracing::{trace, warn};

use crate::{protocol::player_script::PlayerScript, proxy::websocket_proxy::WebSocketProxy};

/// An instance of BulletForceHaxV2. It handles the webrequest and websocket proxies as well as the internal state.
#[derive(Default)]
pub struct BulletForceHax {
    state: Arc<futures_util::lock::Mutex<HaxState>>,
}

impl BulletForceHax {
    pub fn get_state(&self) -> Arc<futures_util::lock::Mutex<HaxState>> {
        self.state.clone()
    }
}

/// The internal state.
#[derive(Default)]
pub struct HaxState {
    // state
    pub global_state: GlobalState,
    pub lobby_state: Option<(WebSocketProxy, LobbyState)>,
    pub gameplay_state: Option<(WebSocketProxy, GameplayState)>,

    // features
    pub show_mobile_games: bool,
    pub show_other_versions: bool,
    pub strip_passwords: bool,
}

/// Game-related state that is kept over the lifetime of the program.
#[derive(Default)]
pub struct GlobalState {
    pub user_id: Option<String>,
    pub version: Option<VersionInfo>,
}

/// State for a given lobby connection
#[derive(Default)]
pub struct LobbyState {}

/// State for a given game connection
#[derive(Default)]
pub struct GameplayState {
    /// the player id
    pub player_id: Option<i32>,

    /// our player's actor id
    pub actor_nr: Option<i32>,

    pub match_manager_view_id: Option<i32>,

    /// The player actors currently in the game.
    ///
    /// Keyed by actor id.
    pub players: IndexMap<i32, PlayerActor>,
}

#[derive(Default, Debug)]
pub struct PlayerActor {
    pub view_id: Option<ViewId>,
    pub user_id: Option<String>,
    pub nickname: Option<String>,
    pub team_number: Option<u8>,

    pub health: Option<f32>,
    pub position: Option<Vector3>,
}

impl PlayerActor {
    pub fn merge_player(&mut self, player: Player) {
        trace!(
            data = format!("{player:?}"),
            "Merging player with actor info"
        );
        if let Some(user_id) = player.user_id {
            self.user_id = Some(user_id);
        }
        if let Some(nickname) = player.nickname {
            self.nickname = Some(nickname);
        }

        if let Some(PhotonDataType::Byte(team_number)) = player.custom_properties.get("teamNumber")
        {
            self.team_number = Some(*team_number);
        }
    }

    pub fn merge_instantiation_data(&mut self, instantiation_data: &InstantiationEventData) {
        if instantiation_data.prefab_name != "PlayerBody" {
            warn!(
                "Tried to merge {} into player, expected PlayerBody",
                instantiation_data.prefab_name
            );
            return;
        }
        trace!(
            data = format!("{instantiation_data:?}"),
            "Merging player with instantiation data"
        );

        self.view_id = Some(instantiation_data.get_view_id());
    }

    pub fn merge_player_script(&mut self, script: &PlayerScript) {
        trace!(
            data = format!("{script:?}"),
            "Merging player with player script"
        );

        self.health = Some(script.health as f32 / 100.0);
        self.position = Some(script.position.clone());
    }
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// The version of the game.
    pub game_version: String,
    /// The version of Photon Unity Networking. This is not the version of the Photon .Net Client Library.
    pub photon_version: String,
}
