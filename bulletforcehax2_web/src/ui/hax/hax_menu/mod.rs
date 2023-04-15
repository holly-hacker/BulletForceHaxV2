use shared::C2SMessage;
use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::ui::hax::{HaxIpcStore, HaxSettingsStore, HaxStateStore};

#[function_component(HaxMenu)]
pub fn hax_menu() -> Html {
    let (hax, _) = use_store::<HaxStateStore>();
    let (settings, settings_dispatch) = use_store::<HaxSettingsStore>();
    let (ipc, _) = use_store::<HaxIpcStore>();
    let hax = hax.0.as_ref().unwrap();
    let settings = settings.0.as_ref().unwrap();
    // let ipc = ipc.0.as_ref().unwrap();

    // TODO: spit up into smaller files so we don't have these large chunks of code here
    let ipc_1 = ipc.clone();
    let callback_show_mobile_games =
        settings_dispatch.reduce_callback_with(move |store, event: bool| {
            let mut new_state = store.0.as_ref().unwrap().clone();
            new_state.show_mobile_games = event;
            ipc_1
                .0
                .as_ref()
                .unwrap()
                .send(C2SMessage::UpdateSettings(new_state.clone()));
            HaxSettingsStore(Some(new_state)).into()
        });
    let ipc_1 = ipc.clone();
    let callback_show_other_versions =
        settings_dispatch.reduce_callback_with(move |store, event: bool| {
            let mut new_state = store.0.as_ref().unwrap().clone();
            new_state.show_other_versions = event;
            ipc_1
                .0
                .as_ref()
                .unwrap()
                .send(C2SMessage::UpdateSettings(new_state.clone()));
            HaxSettingsStore(Some(new_state)).into()
        });
    let ipc_1 = ipc;
    let callback_strip_passwords =
        settings_dispatch.reduce_callback_with(move |store, event: bool| {
            let mut new_state = store.0.as_ref().unwrap().clone();
            new_state.strip_passwords = event;
            ipc_1
                .0
                .as_ref()
                .unwrap()
                .send(C2SMessage::UpdateSettings(new_state.clone()));
            HaxSettingsStore(Some(new_state)).into()
        });

    html! {
        <ybc::Container>
            <ybc::Content>
                <ul>
                    <li>
                        {"User ID: "}
                        {
                            if let Some(user_id) = &hax.global_state.user_id {
                                user_id.clone()
                            } else {
                                "unknown".to_string()
                            }
                        }
                    </li>
                    <li>
                        {"Game version: "}
                        {
                            if let Some(version) = &hax.global_state.version {
                                format!("{} (photon {})", version.game_version, version.photon_version)
                            } else {
                                "unknown".to_string()
                            }
                        }
                    </li>
                </ul>

                <h3>{"Lobby"}</h3>
                <ybc::Control>
                    <ybc::Checkbox name="test" checked={settings.show_mobile_games} update={callback_show_mobile_games}>
                        {"Show mobile games"}
                    </ybc::Checkbox>
                </ybc::Control>

                <ybc::Control>
                    <ybc::Checkbox name="test" checked={settings.show_other_versions} update={callback_show_other_versions}>
                        {"Show versions for other games"}
                    </ybc::Checkbox>
                </ybc::Control>

                <ybc::Control>
                    <ybc::Checkbox name="test" checked=true update={callback_strip_passwords}>
                        {"Strip passwords"}
                    </ybc::Checkbox>
                </ybc::Control>

                <h3>{"Gameplay"}</h3>
                <ybc::Control>
                    <ybc::Input name="" value="" update={|_| {}} />
                </ybc::Control>

                if let Some(state) = &hax.gameplay_state {
                    <h3>{"Info - Players"}</h3>
                    <ybc::Table>
                        <thead>
                            <tr>
                                <th>{"Actor ID"}</th>
                                <th>{"Team"}</th>
                                <th>{"Health"}</th>
                                <th>{"Position"}</th>
                                <th>{"Name"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {state.players.iter().map(|(id, actor)| html!{
                                <tr>
                                    <td><code>{id}</code></td>
                                    <td>{actor.team_number}</td>
                                    <td>{actor.health}</td>
                                    <td><code>{actor.position.clone().map(|x| format!("{:.2}, {:.2}, {:.2}", x.0, x.1, x.2))}</code></td>
                                    <td>{actor.nickname.clone()}</td>
                                </tr>
                            }).collect::<Html>()}
                        </tbody>
                    </ybc::Table>
                }
            </ybc::Content>
        </ybc::Container>
    }
}
