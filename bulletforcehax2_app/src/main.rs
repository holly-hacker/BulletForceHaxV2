#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod asset_server;
mod config;
mod version_manager;

use bulletforcehax2_lib::hax::BulletForceHax;
use bulletforcehax2_ui::BulletForceHaxMenu;
use tao_egui::WindowCreationSettings;
use tracing::{error, info};
use version_manager::VersionManager;
use wry::{
    application::{
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        menu::{MenuBar, MenuItem, MenuItemAttributes, MenuType},
        window::WindowBuilder,
    },
    webview::{WebContext, WebViewBuilder},
};

#[tokio::main]
async fn main() {
    if let Err(err) = real_main().await {
        error!("Fatal error in main: {err:?}");
    }
}

async fn real_main() -> anyhow::Result<()> {
    // read config from cli args
    let config = config::get_config();

    // initialize logging
    let _guard = init_logging(&config);

    // version manager init
    let version_manager = VersionManager::new(&config.game_dir).unwrap();

    let version_info = match version_manager.version() {
        Some(x) => x,
        None => match version_manager.show_version_downloader_blocking().unwrap() {
            Some(x) => x,
            None => return Ok(()),
        },
    };
    info!("Initialized game version");

    asset_server::start_asset_server(version_info, 48897).await;
    info!("Initialized asset server");

    let mut hax = BulletForceHax::default();
    hax.start_webrequest_proxy();
    hax.start_websocket_proxy();
    info!("Initialized hax");

    let state = hax.get_state();

    // create menu structure
    let mut file_submenu = MenuBar::new();
    file_submenu.add_native_item(MenuItem::Quit);

    let mut tools_submenu = MenuBar::new();
    // let hax_menu_item = tools_submenu.add_item(MenuItemAttributes::new("Open Hax Menu"));
    let devtools_menu_item = tools_submenu.add_item(MenuItemAttributes::new("Open Devtools"));

    let mut menu = MenuBar::new();
    menu.add_submenu("File", true, file_submenu);
    menu.add_submenu("Tools", true, tools_submenu);

    // initialize an event loop
    let event_loop = EventLoop::new();

    // create the egui window
    let mut egui_window = tao_egui::TaoEguiWindow::new(
        &event_loop,
        WindowCreationSettings {
            size: (300f32, 600f32),
            window_title: "Hax Menu".into(),
        },
    );

    // create the window for the webview
    let window = WindowBuilder::new()
        .with_title("BulletForceHax")
        .with_menu(menu)
        .build(&event_loop)?;

    // initialize the wry webview
    // https://github.com/rust-lang/rust/issues/92750
    let webview_profile_path = if config.profile_dir.is_absolute() {
        config.profile_dir.clone()
    } else {
        std::env::current_dir().unwrap().join(&config.profile_dir)
    };
    let webview = WebViewBuilder::new(window)?
        .with_web_context(&mut WebContext::new(Some(webview_profile_path)))
        .with_devtools(true)
        .with_url("http://localhost:48897/")?
        .build()?;

    if config.open_devtools {
        webview.open_devtools();
    }

    // initialize the hax menu
    let mut hax_app = BulletForceHaxMenu::new(state);

    // start event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = egui_window
            .handle_event(&event, |ctx| hax_app.update(ctx))
            .unwrap_or(ControlFlow::Wait);

        match event {
            Event::NewEvents(StartCause::Init) => {
                info!("Event loop has started!");
            }
            Event::WindowEvent {
                window_id, event, ..
            } if webview.window().id() == window_id => match &event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            },
            /*
            Event::MenuEvent {
                menu_id,
                origin: MenuType::MenuBar,
                ..
            } if menu_id == hax_menu_item.clone().id() => {
                info!("hax menu button clicked")
            }
            */
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

fn init_logging(config: &config::Config) -> tracing_appender::non_blocking::WorkerGuard {
    use tracing::{level_filters::LevelFilter, Level};
    use tracing_subscriber::prelude::*;

    let logging_level_console = cfg!(debug_assertions)
        .then_some(Level::DEBUG)
        .unwrap_or(Level::INFO);
    let logging_level_file = cfg!(debug_assertions)
        .then_some(Level::TRACE)
        .unwrap_or(Level::DEBUG);

    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("app", Level::TRACE)
        .with_target("bulletforcehax2_lib", Level::TRACE)
        .with_target("bulletforcehax2_ui", Level::TRACE);

    let console_layer = {
        tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
            .with_filter(filter.clone())
            .with_filter(LevelFilter::from_level(logging_level_console))
    };

    // file logs
    let (file_layer, guard) = {
        use time::OffsetDateTime;
        let current_time =
            OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());

        let file_name = format!(
            "log_{:04}{:02}{:02}_{:02}{:02}{:02}.jsonl",
            current_time.year(),
            u8::from(current_time.month()),
            current_time.day(),
            current_time.hour(),
            current_time.minute(),
            current_time.second()
        );

        // we're using tracing_appender because it provides non-blocking logging
        // just logging using std::fs::File may be enough, but has to be tested first.
        let appender = tracing_appender::rolling::never(&config.log_dir, file_name);
        let (non_blocking_appender, guard) = tracing_appender::non_blocking(appender);

        let layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking_appender)
            .json()
            .with_filter(filter)
            .with_filter(LevelFilter::from_level(logging_level_file));

        (layer, guard)
    };

    /*
    let subscriber = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .finish();
    */
    let subscriber = tracing_subscriber::registry();

    let subscriber = subscriber.with(file_layer).with(console_layer);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    #[cfg(debug_assertions)]
    {
        tracing::trace!("trace enabled");
        tracing::debug!("debug enabled");
        tracing::info!("info enabled");
        tracing::warn!("warn enabled");
        tracing::error!("error enabled");
    }

    guard
}
