//! Types that are shared between projects

use indexmap::IndexMap;
use photon_lib::{highlevel::structs::ViewId, primitives::Vector3};
use serde::{Deserialize, Serialize};

/// A message sent from the proxy to the front-end.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum S2CMessage {
    InitialState(HaxStateUpdate, FeatureSettings),
    NewGameState(HaxStateUpdate),
}

/// A message sent from the front-end to the proxy.
#[derive(Serialize, Deserialize, Debug)]
pub enum C2SMessage {
    UpdateSettings(FeatureSettings),
    Command,
}

/// Game state as it is sent over the network.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct HaxStateUpdate {
    pub global_state: GlobalState,
    pub lobby_state: Option<LobbyState>,
    pub gameplay_state: Option<GameplayState>,
}

/// Game-related state that is kept over the lifetime of the program.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct GlobalState {
    pub user_id: Option<String>,
    pub version: Option<VersionInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    /// The version of the game.
    pub game_version: String,
    /// The version of Photon Unity Networking. This is not the version of the Photon .Net Client Library.
    pub photon_version: String,
}

/// State for a given lobby connection
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct LobbyState {}

/// State for a given game connection
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
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

/// The current user-defined settings.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct FeatureSettings {
    pub show_mobile_games: bool,
    pub show_other_versions: bool,
    pub strip_passwords: bool,
    pub spoofed_name: (bool, String),
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct PlayerActor {
    pub view_id: Option<ViewId>,
    pub user_id: Option<String>,
    pub nickname: Option<String>,
    pub team_number: Option<u8>,

    pub health: Option<f32>,
    pub position: Option<Vector3>,
    pub facing_direction: Option<f32>,
}
