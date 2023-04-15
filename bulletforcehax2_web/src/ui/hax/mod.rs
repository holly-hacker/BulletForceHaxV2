mod hax_menu;

use shared::{FeatureSettings, HaxStateUpdate, S2CMessage};
use yew::{function_component, html, use_state, Callback, Html};
use yewdux::{prelude::use_store, store::Store};

use crate::{hax_ipc::HaxIpc, ui::hax::hax_menu::HaxMenu};

/// A store containing the latest [HaxStateUpdate], if any.
#[derive(Default, PartialEq, Store)]
pub struct HaxStateStore(Option<HaxStateUpdate>);

/// A store containing the latest [FeatureSettings], if any.
#[derive(Default, PartialEq, Store)]
pub struct HaxSettingsStore(Option<FeatureSettings>);

#[derive(Default, PartialEq, Store)]
pub struct HaxIpcStore(Option<HaxIpc>);

#[function_component(HaxPage)]
pub fn hax_page() -> Html {
    // supposedly this callback only gets created once because it depends on `()`
    let (state, state_dispatch) = use_store::<HaxStateStore>();
    let (_, settings_dispatch) = use_store::<HaxSettingsStore>();
    let (_, ipc_dispatch) = use_store::<HaxIpcStore>();
    _ = use_state(|| {
        let state_dispatch = state_dispatch.clone();
        let settings_dispatch = settings_dispatch.clone();

        let callback = Callback::from(move |msg: S2CMessage| match msg {
            S2CMessage::InitialState(game, settings) => {
                state_dispatch.set(HaxStateStore(Some(game)));
                settings_dispatch.set(HaxSettingsStore(Some(settings)));
            }
            S2CMessage::NewGameState(game) => {
                state_dispatch.set(HaxStateStore(Some(game)));
            }
        });
        let ipc = HaxIpc::connect(callback);
        ipc_dispatch.set(HaxIpcStore(Some(ipc)));
    });

    html! {
        <>
            if state.0.is_some() {
                <HaxMenu />
            }
        </>
    }
}
