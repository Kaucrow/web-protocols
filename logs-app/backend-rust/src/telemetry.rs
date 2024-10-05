//! Logging instrumentation setup.

use tracing_subscriber::{fmt::{self, time::FormatTime}, layer::SubscriberExt, EnvFilter, Layer};
use tracing_appender;
use chrono::{Datelike, Timelike};

/// Log timestamp formatter, with the format `[day-month-year] [hour:minute:second.nanosecond]`.
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
pub fn get_subscriber() -> (impl tracing::Subscriber + Send + Sync, tracing_appender::non_blocking::WorkerGuard) {
    let file_appender = tracing_appender::rolling::never("logs", "log.txt");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let console_filter = "debug,h2=info".to_string();
    let console_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(console_filter));

    let file_filter = EnvFilter::new("backend-file=trace");

    let subscriber = tracing_subscriber::Registry::default()
        .with(fmt::layer()
            .with_writer(std::io::stdout)
            .with_filter(console_filter))
        .with(fmt::layer()
            .with_writer(non_blocking)
            .with_timer(TimeFormat)
            .with_ansi(false)
            .with_filter(file_filter));

    (subscriber, guard)
}

/// Set the tracing subscriber.
pub fn init_subscriber(subscriber: impl tracing::Subscriber + Send + Sync) {
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}