//! The main module of BulletForceHaxV2.

mod hax_impl;
mod impl_webrequest;
mod impl_websocket;

use std::sync::Arc;

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

/// The internal state.
#[derive(Default)]
pub struct HaxState {
    // socket info
    pub lobby_socket: Option<WebSocketProxy>,
    pub gameplay_socket: Option<WebSocketProxy>,

    // info
    pub user_id: Option<String>,
    pub game_version: Option<String>,

    // features
    pub show_mobile_games: bool,
    pub show_other_versions: bool,
    pub strip_passwords: bool,
}

impl BulletForceHax {
    pub fn get_state(&self) -> Arc<futures_util::lock::Mutex<HaxState>> {
        self.state.clone()
    }
}
