use std::{sync::Arc, time::Duration};

use bulletforcehax2_lib::hax::HaxState;
use bulletforcehax2_lib::indexmap::indexmap;
use futures_util::lock::Mutex;
use photon_lib::{
    photon_data_type::PhotonDataType,
    photon_message::{EventData, PhotonMessage},
    realtime::{
        constants::{event_code, parameter_code},
        PhotonMapConversion, RoomInfo,
    },
};

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

            let mut hax = futures::executor::block_on(self.hax.lock());

            #[cfg(debug_assertions)]
            {
                ui.label(format!("lobby socket: {}", hax.lobby_socket.is_some()));
                ui.label(format!(
                    "gameplay socket: {}",
                    hax.gameplay_socket.is_some()
                ));
            }

            ui.add_space(16f32);
            ui.heading("Lobby");
            ui.checkbox(&mut hax.show_mobile_games, "Show mobile games");

            ui.add_space(16f32);

            drop(hax);
            if let Some(fps) = frame.info().cpu_usage {
                ui.label(format!("cpu usage {:?}", Duration::from_secs_f32(fps)));
            }
        });
    }
}
