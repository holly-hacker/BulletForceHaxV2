use std::sync::Arc;

use bulletforcehax2_lib::hax::HaxState;
use egui::RichText;
use egui_extras::{Size, TableBuilder};
use futures_util::lock::Mutex;

pub struct BulletForceHaxMenu {
    hax: Arc<Mutex<HaxState>>,
    first_frame: bool,
}

impl BulletForceHaxMenu {
    pub fn new(hax: Arc<Mutex<HaxState>>) -> Self {
        Self {
            hax,
            first_frame: true,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        if self.first_frame {
            ctx.set_pixels_per_point(1.5f32);
            self.first_frame = false;
        }

        // set framerate to 20fps
        ctx.request_repaint_after(std::time::Duration::from_secs_f32(1f32 / 10f32));

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut hax = futures::executor::block_on(self.hax.lock());

            ui.heading("Info");
            if let Some(user_id) = &hax.user_id {
                ui.label(format!("User ID: {user_id}"));
            }
            if let Some(version) = &hax.game_version {
                ui.label(format!("Game version: {version}"));
            }
            ui.add_space(16f32);

            ui.heading("Lobby");
            ui.checkbox(&mut hax.show_mobile_games, "Show mobile games");
            ui.checkbox(
                &mut hax.show_other_versions,
                "Show games for other versions",
            );
            ui.checkbox(&mut hax.strip_passwords, "Strip passwords");
            ui.add_space(16f32);

            ui.heading("UI");
            ui.horizontal(|ui| {
                let scale = ctx.pixels_per_point();
                ui.label(format!("Zoom: {scale}"));

                if ui.button("-").clicked() {
                    if scale > 2f32 {
                        ctx.set_pixels_per_point(scale - 1f32);
                    } else if scale > 1f32 {
                        ctx.set_pixels_per_point(scale - 0.5f32);
                    }
                }
                if ui.button("+").clicked() {
                    if scale < 2f32 {
                        ctx.set_pixels_per_point(scale + 0.5f32);
                    } else {
                        ctx.set_pixels_per_point(scale + 1f32);
                    }
                }
            });
            ui.add_space(16f32);

            if hax.gameplay_socket.is_some() {
                ui.heading("Info - Players");
                {
                    TableBuilder::new(ui)
                        .striped(true)
                        .column(Size::initial(45.0))
                        .column(Size::remainder())
                        .resizable(true)
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.label(RichText::new("ActorId").strong());
                            });
                            header.col(|ui| {
                                ui.label(RichText::new("Name").strong());
                            });
                        })
                        .body(|mut body| {
                            for (actor_id, player) in &hax.players {
                                body.row(18.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(actor_id.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(match &player.nickname {
                                            Some(x) => x.as_str(),
                                            None => "",
                                        });
                                    });
                                });
                            }
                        });
                }
                ui.add_space(16f32);
            }

            #[cfg(debug_assertions)]
            {
                ui.heading("Debug");
                ui.label(format!("lobby socket: {}", hax.lobby_socket.is_some()));
                ui.label(format!(
                    "gameplay socket: {}",
                    hax.gameplay_socket.is_some()
                ));
                ui.add_space(16f32);
            }

            drop(hax);

            // TODO: add back FPS counter
            ui.label(format!("Time: {}", ctx.input().time));
        });
    }
}
