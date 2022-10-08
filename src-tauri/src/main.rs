#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod version_manager;

use std::{path::Path, sync::Arc};

use bulletforcehax2_lib::hax::{BulletForceHax, HaxState};
use bulletforcehax2_ui::BulletForceHaxMenu;
use futures_util::lock::Mutex;
use once_cell::sync::OnceCell;
use tauri::{
    http::{Request, Response, ResponseBuilder},
    AppHandle, CustomMenuItem, Manager, Menu, MenuItem, Submenu,
};
use tracing::{debug, info};
use tracing_subscriber::prelude::*;
use version_manager::{VersionConfig, VersionManager};

static GAME_VERSION: OnceCell<VersionConfig> = OnceCell::new();

fn bulletforce_handler(
    _handle: &AppHandle,
    request: &Request,
) -> Result<Response, Box<dyn std::error::Error>> {
    let mut path = &request.uri()["bulletforce://".len()..];

    if path.starts_with("localhost/") {
        path = &path["localhost/".len()..];
    }

    debug!("protocol req: {}", path);

    let version = GAME_VERSION.get().unwrap();
    let path = match path {
        "Build/$$game$$.json" => version.get_game_json(),
        "$$loader$$.js" => version.get_unity_loader(),
        _ => version.get_path(path),
    };
    let content = std::fs::read(path).unwrap();

    let builder = ResponseBuilder::new();
    builder
        .status(200)
        .header("Access-Control-Allow-Origin", "*")
        .body(content)
}

#[tokio::main]
async fn main() {
    // initialize logging
    let default_logging_level = tracing::Level::DEBUG;
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(default_logging_level)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .finish();

    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("app", default_logging_level)
        .with_target("bulletforcehax2_lib", default_logging_level)
        .with_target("bulletforcehax2_ui", default_logging_level);

    subscriber.with(filter).init();

    #[cfg(debug_assertions)]
    {
        tracing::trace!("trace enabled");
        tracing::debug!("debug enabled");
        tracing::info!("info enabled");
        tracing::warn!("warn enabled");
        tracing::error!("error enabled");
    }

    // version manager init
    let version_manager = VersionManager::new(Path::new("bfhax_data")).unwrap();

    // NOTE: this does not use the tauri lifecycle management
    let version_info = match version_manager.version() {
        Some(x) => x,
        None => match version_manager.show_version_downloader_blocking().unwrap() {
            Some(x) => x,
            None => return,
        },
    };

    GAME_VERSION.set(version_info).ok().unwrap();
    info!("Initialized game version global");

    let mut hax = BulletForceHax::default();
    hax.start_webrequest_proxy();
    hax.start_websocket_proxy();
    info!("Initialized hax");

    let state = hax.get_state();

    // create menu
    let file_submenu = Submenu::new("File", Menu::new().add_native_item(MenuItem::Quit));
    let menu = Menu::new()
        .add_submenu(file_submenu)
        .add_item(CustomMenuItem::new("show_menu", "Menu"));

    // create tauri app and block on it
    // when the tauri app closes, exit from main
    #[allow(clippy::single_match)]
    tauri::Builder::default()
        .setup(|app| {
            app.wry_plugin(tauri_egui::EguiPluginBuilder::new(app.handle()));

            // automatically open devtools on debug builds
            #[cfg(debug_assertions)]
            {
                use tauri::Manager;
                app.get_window("main").unwrap().open_devtools();
            }
            Ok(())
        })
        .manage(state)
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            "show_menu" => {
                let app = event.window().app_handle();
                let state: tauri::State<Arc<Mutex<HaxState>>> = app.state();
                let state = state.inner().clone();

                info!("Opening hax ui");
                app.state::<tauri_egui::EguiPluginHandle>()
                    .create_window(
                        "hax".to_string(),
                        Box::new(move |_cc| Box::new(BulletForceHaxMenu::new(state))),
                        "Hax Menu".into(),
                        tauri_egui::eframe::NativeOptions {
                            initial_window_size: Some((320f32, 640f32).into()),
                            ..Default::default()
                        },
                    )
                    .unwrap();
            }
            _ => (),
        })
        .register_uri_scheme_protocol("bulletforce", bulletforce_handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
