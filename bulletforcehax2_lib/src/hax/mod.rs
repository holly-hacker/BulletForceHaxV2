//! The main module of BulletForceHaxV2.

mod hax_impl;
mod impl_webrequest;
mod impl_websocket;

use std::sync::Arc;

use photon_lib::{highlevel::structs::Player, indexmap::IndexMap};

use crate::proxy::websocket_proxy::WebSocketProxy;

/// An instance of BulletForceHaxV2. It handles the webrequest and websocket proxies as well as the internal state.
///
/// # Remarks
/// It is not recommended to create multiple instances of this because [BulletForceHax::start_websocket_proxy] and
/// [BulletForceHax::start_webrequest_proxy] can only be run once, as they bind to a pre-defined port.
#[derive(Default)]
pub struct BulletForceHax {
    webrequest_proxy: Option<()>,
    websocket_proxy: Option<()>,
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
    pub player_id: Option<i32>,
    pub players: IndexMap<i32, Player>,
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// The version of the game.
    pub game_version: String,
    /// The version of Photon Unity Networking. This is not the version of the Photon .Net Client Library.
    pub photon_version: String,
}
