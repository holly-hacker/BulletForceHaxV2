use std::{sync::Arc, time::Duration};

use bulletforcehax2_lib::hax::HaxState;
use futures_util::lock::Mutex;

pub struct BulletForceHaxMenu {
    hax: Arc<Mutex<HaxState>>,
}

impl BulletForceHaxMenu {
    pub fn new(hax: Arc<Mutex<HaxState>>) -> Self {
        Self { hax }
    }
}

impl eframe::App for BulletForceHaxMenu {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // set framerate to 20fps
        ctx.request_repaint_after(std::time::Duration::from_secs_f32(1f32 / 20f32));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("hax menu goes here :D");

            if true {
                let ret = futures::executor::block_on(self.hax.lock());
                ui.label(format!("lobby socket: {}", ret.lobby_socket.is_some()));
                ui.label(format!(
                    "gameplay socket: {}",
                    ret.gameplay_socket.is_some()
                ));
            }

            if let Some(fps) = frame.info().cpu_usage {
                ui.label(format!("cpu usage {:?}", Duration::from_secs_f32(fps)));
            }
        });
    }
}
