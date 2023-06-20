use egui::{Color32, RichText};

use crate::ui_context::UiContext;

pub fn render_ui(egui_ctx: &egui::Context, ctx: &mut UiContext) {
    egui::SidePanel::left("left_panel").show(egui_ctx, |ui| {
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

        ui.add_space(16.);
        ui.label(RichText::new("Connected to game").color(Color32::WHITE));
        ui.add_space(16.);

        if let Some(version) = &ctx.state.global_state.version {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Game version:").strong());
                ui.label(&version.game_version);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Photon version:").strong());
                ui.label(&version.photon_version);
            });
        }
        if let Some(user_id) = &ctx.state.global_state.user_id {
            ui.horizontal(|ui| {
                ui.label(RichText::new("User ID:").strong());
                ui.label(user_id);
            });
        }
    });

    egui::Window::new("Settings").show(egui_ctx, |ui| {
        ui.heading("Lobby");
        if ui
            .checkbox(&mut ctx.settings.show_mobile_games, "Show mobile games")
            .changed()
        {
            ctx.mark_settings_dirty();
        }
        if ui
            .checkbox(
                &mut ctx.settings.show_other_versions,
                "Show games from other versions",
            )
            .changed()
        {
            ctx.mark_settings_dirty();
        }
        if ui
            .checkbox(&mut ctx.settings.strip_passwords, "Strip passwords")
            .changed()
        {
            ctx.mark_settings_dirty();
        }

        ui.heading("Gameplay");
        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut ctx.settings.spoofed_name.0, "Spoof name")
                .changed()
            {
                ctx.mark_settings_dirty();
            }
            ui.add_enabled_ui(ctx.settings.spoofed_name.0, |ui| {
                if ui
                    .text_edit_singleline(&mut ctx.settings.spoofed_name.1)
                    .changed()
                {
                    ctx.mark_settings_dirty();
                }
            });
        });
    });

    egui::Window::new("Debug").show(egui_ctx, |ui| {
        ui.code(format!("{:#?}", ctx.state));
        ui.separator();
        ui.code(format!("{:#?}", ctx.settings));
    });
}
