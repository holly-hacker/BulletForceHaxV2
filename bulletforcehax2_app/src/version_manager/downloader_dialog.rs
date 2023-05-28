use std::collections::HashMap;

use anyhow::Result;
use bulletforcehax2_lib::version_scraper::*;
use bytesize::ByteSize;
use eframe::{
    egui::{self, Color32, ProgressBar, RichText},
    App,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tracing::{debug, error, info};

pub struct DownloaderDialog {
    tx: Sender<Vec<DownloadedFile>>,
    rx_scraper: Option<Receiver<ProgressReport>>,
    file_progress: HashMap<String, FileDownloadProgress>,
    files: Vec<DownloadedFile>,
    error: Option<String>,
}

#[derive(Clone)]
pub struct DownloadedFile {
    pub name: String,
    pub file_type: FileType,
    pub data: Vec<u8>,
}

struct FileDownloadProgress {
    downloaded: u64,
    total: Option<u64>,
    finished: bool,
}

impl DownloaderDialog {
    pub fn new() -> (Self, Receiver<Vec<DownloadedFile>>) {
        let (tx, rx) = channel(4);
        (
            Self {
                tx,
                rx_scraper: None,
                file_progress: HashMap::new(),
                files: vec![],
                error: None,
            },
            rx,
        )
    }

    pub fn show_dialog() -> Result<Vec<DownloadedFile>> {
        let (dialog, mut rx) = Self::new();

        debug!("dialog starting");
        eframe::run_native(
            "Version Downloader",
            eframe::NativeOptions {
                initial_window_size: Some((480.0, 240.0).into()),
                ..Default::default()
            },
            Box::new(|_| Box::new(dialog)),
        );

        debug!("dialog done");

        let recv = rx.try_recv()?;
        debug!("dialog response has len {}", recv.len());

        Ok(recv)
    }
}

impl App for DownloaderDialog {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        // re-draw continuously to make sure the channel reader runs
        ctx.request_repaint();

        let rx_scraper = self
            .rx_scraper
            .get_or_insert_with(|| start_download_thread().unwrap());

        match rx_scraper.try_recv() {
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                tracing::debug!("Channel disconnected")
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => (),
            Ok(report) => {
                match report {
                    ProgressReport::Progress {
                        file_type: _,
                        name,
                        downloaded,
                        total,
                    } => match self.file_progress.get_mut(&name) {
                        Some(file) => {
                            file.downloaded = downloaded;
                            file.total = total;
                        }
                        None => {
                            self.file_progress.insert(
                                name,
                                FileDownloadProgress {
                                    downloaded,
                                    total,
                                    finished: false,
                                },
                            );
                        }
                    },
                    ProgressReport::FileDownloaded {
                        file_type,
                        name,
                        data,
                    } => {
                        info!("File downloaded: {name}");
                        // insert into state
                        match self.file_progress.get_mut(&name) {
                            Some(file) => {
                                file.finished = true;
                            }
                            None => {
                                self.file_progress.insert(
                                    name.clone(),
                                    FileDownloadProgress {
                                        downloaded: data.len() as u64,
                                        total: Some(data.len() as u64),
                                        finished: true,
                                    },
                                );
                            }
                        };

                        self.files.push(DownloadedFile {
                            name,
                            file_type,
                            data,
                        });
                    }
                    ProgressReport::AllFilesDownloaded => {
                        info!("Finished downloading all files, closing dialog");
                        // NOTE: not sure if I want to block here, it's probably fine
                        futures::executor::block_on(self.tx.send(self.files.clone()))
                            .ok()
                            .unwrap();
                        frame.close()
                    }
                    ProgressReport::Crashed(e) => {
                        error!("Received crash message: {e}");
                        self.error = Some(e)
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Dowloading game...");

            for (name, info) in self.file_progress.iter() {
                ui.horizontal(|ui| {
                    if info.finished {
                        ui.add(
                            ProgressBar::new(1.0)
                                .show_percentage()
                                .desired_width(100f32),
                        );
                    } else if let Some(total) = info.total {
                        ui.add(
                            ProgressBar::new((info.downloaded as f32) / (total as f32))
                                .show_percentage()
                                .desired_width(100f32),
                        );
                    } else {
                        ui.spinner();
                    }

                    match info.total {
                        Some(total) => ui.label(format!(
                            "{name} ({}/{} downloaded)",
                            ByteSize(info.downloaded),
                            ByteSize(total)
                        )),
                        None => {
                            ui.label(format!("{name} ({} downloaded)", ByteSize(info.downloaded)))
                        }
                    }
                });
            }

            if let Some(error) = &self.error {
                ui.add_space(8.0);
                ui.heading(
                    RichText::new("An error occurred!")
                        .color(Color32::RED)
                        .heading(),
                );
                ui.label(error);
            }
        });
    }
}
