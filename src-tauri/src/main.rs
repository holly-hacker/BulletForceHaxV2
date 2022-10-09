#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod version_manager;

use std::path::{Path, PathBuf};

use bulletforcehax2_lib::hax::BulletForceHax;
use once_cell::sync::OnceCell;
use tracing::{debug, error, info};
use tracing_subscriber::prelude::*;
use version_manager::{VersionConfig, VersionManager};
use wry::{
    application::{
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        menu::{MenuBar, MenuItem, MenuItemAttributes, MenuType},
        window::WindowBuilder,
    },
    http::{Request, Response, ResponseBuilder},
    webview::{WebContext, WebViewBuilder},
};

static GAME_VERSION: OnceCell<VersionConfig> = OnceCell::new();

fn static_file_handler(request: &Request) -> Result<Response, wry::Error> {
    let mut path = &request.uri()["static://".len()..];

    if path.starts_with("localhost/") {
        path = &path["localhost/".len()..];
    }

    let content = match path {
        "" => Some(include_bytes!("../ui/index.html").to_vec()),
        _ => None,
    };

    let builder = ResponseBuilder::new();

    if let Some(content) = content {
        builder
            .status(200)
            .header("Access-Control-Allow-Origin", "*")
            .body(content.to_vec())
            .map_err(Into::into)
    } else {
        builder
            .status(404)
            .body(b"not found".to_vec())
            .map_err(Into::into)
    }
}

fn bulletforce_handler(request: &Request) -> Result<Response, wry::Error> {
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
        .map_err(Into::into)
}

#[tokio::main]
async fn main() {
    if let Err(err) = real_main().await {
        error!("Fatal error in main: {err:?}");
    }
}

async fn real_main() -> anyhow::Result<()> {
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

    let version_info = match version_manager.version() {
        Some(x) => x,
        None => match version_manager.show_version_downloader_blocking().unwrap() {
            Some(x) => x,
            None => return Ok(()),
        },
    };

    GAME_VERSION.set(version_info).ok().unwrap();
    info!("Initialized game version global");

    let mut hax = BulletForceHax::default();
    hax.start_webrequest_proxy();
    hax.start_websocket_proxy();
    info!("Initialized hax");

    let _state = hax.get_state();

    // create menu structure
    let mut file_submenu = MenuBar::new();
    file_submenu.add_native_item(MenuItem::Quit);

    let mut tools_submenu = MenuBar::new();
    let hax_menu_item = tools_submenu.add_item(MenuItemAttributes::new("Open Hax Menu"));
    let devtools_menu_item = tools_submenu.add_item(MenuItemAttributes::new("Open Devtools"));

    let mut menu = MenuBar::new();
    menu.add_submenu("File", true, file_submenu);
    menu.add_submenu("Tools", true, tools_submenu);

    // initialize wry webview
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("BulletForceHax")
        .with_menu(menu)
        .build(&event_loop)?;
    let webview = WebViewBuilder::new(window)?
        .with_web_context(&mut WebContext::new(Some(
            std::env::current_dir()
                .unwrap()
                .join("./bfhax_data/webview_data_directory"),
        )))
        .with_custom_protocol("static".into(), static_file_handler)
        .with_custom_protocol("bulletforce".into(), bulletforce_handler)
        .with_devtools(true)
        .with_url("static://localhost/")?
        .build()?;

    // start event loop
    event_loop.run(move |event, _event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                info!("Wry has started!");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MenuEvent {
                menu_id,
                origin: MenuType::MenuBar,
                ..
            } if menu_id == hax_menu_item.clone().id() => {
                info!("hax menu button clicked");
            }
            Event::MenuEvent {
                menu_id,
                origin: MenuType::MenuBar,
                ..
            } if menu_id == devtools_menu_item.clone().id() => {
                webview.open_devtools();
            }
            _ => (),
        }
    });
}
