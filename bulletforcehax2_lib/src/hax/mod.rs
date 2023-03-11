//! The main module of BulletForceHaxV2.

mod hax_impl;
mod impl_proxy;

use std::sync::Arc;

use photon_lib::{
    highlevel::structs::{InstantiationEventData, Player},
    photon_data_type::PhotonDataType,
};
use shared::{FeatureState, GameplayState, GlobalState, HaxStateNetwork, LobbyState, PlayerActor};
use tracing::{trace, warn};

use crate::{protocol::player_script::PlayerScript, proxy::websocket_proxy::WebSocketProxy};

/// Exclusive, thread-safe access to [HaxState].
pub type HaxSharedState = Arc<futures_util::lock::Mutex<HaxState>>;

/// An instance of BulletForceHaxV2. It handles the webrequest and websocket proxies as well as the internal state.
#[derive(Default)]
pub struct BulletForceHax {
    state: HaxSharedState,
}

impl BulletForceHax {
    pub fn get_state(&self) -> HaxSharedState {
        self.state.clone()
    }
}

/// The internal state.
#[derive(Default)]
pub struct HaxState {
    pub global_state: GlobalState,
    pub features: FeatureState,
    pub lobby_state: Option<(WebSocketProxy, LobbyState)>,
    pub gameplay_state: Option<(WebSocketProxy, GameplayState)>,
}

impl HaxState {
    pub fn copy_to_network_state(&self) -> HaxStateNetwork {
        // NOTE: doing this entire copy each time we send state isn't very efficient, since it's not needed.
        // we're only doing it because we need to filter out the websocket proxy objects
        HaxStateNetwork {
            global_state: self.global_state.clone(),
            features: self.features.clone(),
            lobby_state: self.lobby_state.as_ref().map(|x| x.1.clone()),
            gameplay_state: self.gameplay_state.as_ref().map(|x| x.1.clone()),
        }
    }
}

pub fn merge_player(actor: &mut PlayerActor, player: &Player) {
    trace!(
        data = format!("{player:?}"),
        "Merging player with actor info"
    );
    if let Some(user_id) = &player.user_id {
        actor.user_id = Some(user_id.clone());
    }
    if let Some(nickname) = &player.nickname {
        actor.nickname = Some(nickname.clone());
    }

    if let Some(PhotonDataType::Byte(team_number)) = player.custom_properties.get("teamNumber") {
        actor.team_number = Some(*team_number);
    }
}

pub fn merge_instantiation_data(
    actor: &mut PlayerActor,
    instantiation_data: &InstantiationEventData,
) {
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

    actor.view_id = Some(instantiation_data.get_view_id());
}

pub fn merge_player_script(actor: &mut PlayerActor, script: &PlayerScript) {
    trace!(
        data = format!("{script:?}"),
        "Merging player with player script"
    );

    actor.health = Some(script.health as f32 / 100.0);
    actor.position = Some(script.position.clone());
    actor.facing_direction = Some(script.move_angle as f32 / 10.0);
}
