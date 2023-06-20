use std::time::Duration;

use egui::{Color32, RichText};

use crate::hax_ipc::HaxIpc;

#[derive(Default)]
pub struct BulletForceHaxApp {
    ipc: Option<HaxIpc>,
}

impl BulletForceHaxApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            ipc: HaxIpc::try_connect(),
        }
    }
}

impl eframe::App for BulletForceHaxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(ipc) = &mut self.ipc {
            ipc.recv_comms();
        }

        let mut ui_context_val = self.ipc.as_ref().and_then(HaxIpc::get_ui_context);

        // create var as mut, since this gets referenced in the UI closures
        let ui_context = &mut ui_context_val;

        let mut try_connect_ipc = false;

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("BulletForceHax");
            egui::warn_if_debug_build(ui);
            if ui.link("Open game").clicked() {
                ui.ctx().output_mut(|o| {
                    o.open_url = Some(egui::output::OpenUrl {
                        url: "/game".into(),
                        new_tab: true,
                    });
                });
            }

            if let Some(ui_ctx) = ui_context {
                ui.add_space(16.);
                ui.label(RichText::new("Connected to game").color(Color32::WHITE));
                ui.add_space(16.);

                if let Some(version) = &ui_ctx.state.global_state.version {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Game version:").strong());
                        ui.label(&version.game_version);
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Photon version:").strong());
                        ui.label(&version.photon_version);
                    });
                }
                if let Some(user_id) = &ui_ctx.state.global_state.user_id {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("User ID:").strong());
                        ui.label(user_id);
                    });
                }
            } else {
                ui.label("IPC not connected, local server dead?");
                if ui.button("Reconnect").clicked() {
                    try_connect_ipc = true;
                }
            }
        });

        if let Some(ui_ctx) = ui_context {
            egui::Window::new("Settings").show(ctx, |ui| {
                ui.heading("Lobby");
                if ui
                    .checkbox(&mut ui_ctx.settings.show_mobile_games, "Show mobile games")
                    .changed()
                {
                    ui_ctx.mark_settings_dirty();
                }
                if ui
                    .checkbox(
                        &mut ui_ctx.settings.show_other_versions,
                        "Show games from other versions",
                    )
                    .changed()
                {
                    ui_ctx.mark_settings_dirty();
                }
                if ui
                    .checkbox(&mut ui_ctx.settings.strip_passwords, "Strip passwords")
                    .changed()
                {
                    ui_ctx.mark_settings_dirty();
                }

                ui.heading("Gameplay");
                ui.horizontal(|ui| {
                    if ui
                        .checkbox(&mut ui_ctx.settings.spoofed_name.0, "Spoof name")
                        .changed()
                    {
                        ui_ctx.mark_settings_dirty();
                    }
                    ui.add_enabled_ui(ui_ctx.settings.spoofed_name.0, |ui| {
                        if ui
                            .text_edit_singleline(&mut ui_ctx.settings.spoofed_name.1)
                            .changed()
                        {
                            ui_ctx.mark_settings_dirty();
                        }
                    });
                });
            });

            egui::Window::new("Debug").show(ctx, |ui| {
                ui.code(format!("{:#?}", ui_ctx.state));
                ui.separator();
                ui.code(format!("{:#?}", ui_ctx.settings));
            });
        }

        if let Some(ipc) = &self.ipc {
            if let Some(mut ui_ctx) = ui_context_val {
                if ui_ctx.is_settings_dirty() {
                    // transmit new state
                    ipc.send_updated_features(&ui_ctx.settings);

                    // overwrite our internal state. we don't get a new copy every time we write it.
                    self.ipc.as_mut().unwrap().state.as_mut().unwrap().1 = ui_ctx.settings;
                }
            }
        }

        if try_connect_ipc {
            self.ipc = HaxIpc::try_connect();
        }

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
