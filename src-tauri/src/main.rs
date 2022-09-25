#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod version_manager;
mod webrequest_proxy;

use std::path::Path;

use once_cell::sync::OnceCell;
use tauri::{
    http::{Request, Response, ResponseBuilder},
    AppHandle, Manager,
};
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

    println!("protocol req: {}", path);

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
    // version manager init
    let version_manager = VersionManager::new(Path::new("bfhax_data")).unwrap();

    let version_info = match version_manager.version() {
        Some(x) => x,
        None => match version_manager.show_version_downloader_blocking().unwrap() {
            Some(x) => x,
            None => return,
        },
    };

    _ = GAME_VERSION.set(version_info);
    println!("Set game version global");

    // set up web proxy
    tokio::spawn(async move {
        webrequest_proxy::block_on_server().await;
    });

    // TODO: set up web socket proxy

    // create tauri app and block on it
    // when the tauri app closes, exit from main
    tauri::Builder::default()
        .setup(|_app| {
            // app.wry_plugin(tauri_egui::EguiPluginBuilder::new(app.handle()));

            // automatically open devtools on debug builds
            #[cfg(debug_assertions)]
            _app.get_window("main").unwrap().open_devtools();
            Ok(())
        })
        .register_uri_scheme_protocol("bulletforce", bulletforce_handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
