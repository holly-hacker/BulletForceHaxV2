use std::time::Duration;

use crate::{hax_ipc::HaxIpc, ui::render_ui};

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

        if let Some(ui_ctx) = ui_context {
            render_ui(ctx, ui_ctx);
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("No connection with game. Did server die?");
                ui.heading("Consider reloading the page.");
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

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
