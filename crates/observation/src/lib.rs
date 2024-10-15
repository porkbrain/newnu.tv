use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

/// Starts a global subscriber that
/// - pretty prints to stderr
/// - logs JSON lines to a rolling file hourly
///
/// Will create the directory if it doesn't exist.
///
/// You want to keep the [WorkerGuard] alive for the duration of the program
/// otherwise logs will not be written to the file.
pub fn with_rolling_logs(rolling_log_dir: impl AsRef<Path>) -> WorkerGuard {
    color_eyre::install().ok();

    let (file_writer, file_guard) = tracing_appender::non_blocking(
        tracing_appender::rolling::hourly(rolling_log_dir, "rolling.log"),
    );

    let registry = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            fmt::layer()
                .pretty()
                .with_file(false)
                .with_line_number(true)
                .with_writer(std::io::stderr),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(file_writer)
                .with_thread_names(true),
        );

    tracing::subscriber::set_global_default(registry).ok();

    file_guard
}
