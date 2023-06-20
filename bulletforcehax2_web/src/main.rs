mod app;
mod hax_ipc;
mod ui;
mod ui_context;

/// Native constructor so `cargo check` does not fail.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "BulletForceHax",
        native_options,
        Box::new(|cc| Box::new(app::BulletForceHaxApp::new(cc))),
    )
}

/// Main entrypoint for web
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(app::BulletForceHaxApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
