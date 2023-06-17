use std::time::Duration;

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
            ipc.do_communication_tick();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("BulletForceHax");
            if ui.link("Open game").clicked() {
                ui.ctx().output_mut(|o| {
                    o.open_url = Some(egui::output::OpenUrl {
                        url: "/game".into(),
                        new_tab: true,
                    });
                });
            }

            if let Some(ipc) = &mut self.ipc {
                ui.label("IPC connected!");
                if let Some(state) = &ipc.state {
                    ui.spacing();
                    ui.code(format!("{:#?}", state.0));
                    ui.spacing();
                    ui.code(format!("{:#?}", state.1));
                } else {
                    ui.label("No state found");
                }
            } else {
                ui.label("IPC not connected");
                if ui.button("Connect").clicked() {
                    self.ipc = HaxIpc::try_connect();
                }
            }

            egui::warn_if_debug_build(ui);
        });

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
