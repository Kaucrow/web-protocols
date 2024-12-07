use crate::prelude::*;
use tracing_subscriber::{
    fmt::{ self, time::FormatTime },
    layer::SubscriberExt,
    EnvFilter,
    Layer
};
use tracing_appender;
use chrono::{ Datelike, Timelike };
use anyhow::Result;

/// Log timestamp formatter, with the format `[day-month-year] [hour:minute:second.nanosecond]`.
#[derive(Clone)]
struct TimeFormat;

impl FormatTime for TimeFormat {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Local::now();

        let (year, month, day, hour, minute, second, nano) =
            (now.year(), now.month(), now.day(),
             now.hour(), now.minute(), now.second(),
             now.timestamp_subsec_nanos());
        
        write!(w, "{}-{}-{} {:02}:{:02}:{:02}.{}", day, month, year, hour, minute, second, nano)
    }
}

/// Build a tracing subscriber.
pub async fn get_subscriber() -> Result<(impl tracing::Subscriber + Send + Sync, tracing_appender::non_blocking::WorkerGuard)> {
    let path = PathBuf::from("./log/backend.log");

    if path.exists() && path.is_file() {
        fs::remove_file(&path).await?
    }

    let file_appender = tracing_appender::rolling::never("log", "backend.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let console_filter = "debug,h2=info".to_string();
    let console_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(console_filter));

    let file_filter = EnvFilter::new("debug,h2=info");

    let subscriber = tracing_subscriber::Registry::default()
        .with(fmt::layer()
            .with_target(false)
            .with_writer(non_blocking)
            .with_timer(TimeFormat)
            .with_ansi(false)
            .with_filter(file_filter))
        .with(fmt::layer()
            .with_target(false)
            .with_writer(std::io::stdout)
            .with_ansi(true)
            .with_filter(console_filter));
        
    Ok((subscriber, guard))
}

/// Set the tracing subscriber.
pub fn init_subscriber(subscriber: impl tracing::Subscriber + Send + Sync) {
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}