pub fn init_logging(config: &crate::config::Config) -> tracing_appender::non_blocking::WorkerGuard {
    use tracing::{level_filters::LevelFilter, Level};
    use tracing_subscriber::prelude::*;

    let logging_level_console = cfg!(debug_assertions)
        .then_some(Level::DEBUG)
        .unwrap_or(Level::INFO);
    let logging_level_file = cfg!(debug_assertions)
        .then_some(Level::TRACE)
        .unwrap_or(Level::DEBUG);

    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("server", Level::TRACE)
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
