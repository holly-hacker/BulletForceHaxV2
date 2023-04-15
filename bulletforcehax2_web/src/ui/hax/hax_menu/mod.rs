use std::rc::Rc;

use shared::HaxStateUpdate;
use yew::prelude::*;

#[function_component(HaxMenu)]
pub fn hax_menu() -> Html {
    let hax = use_context::<Rc<HaxStateUpdate>>().expect("get state update");
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
                {"TODO: add options"}

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
