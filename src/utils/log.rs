// utils/log.rs

use std::env;
use std::time::Instant;

use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::EnvFilter;

/// # Uptime struct for timestamp formatting in logs
struct Uptime(Instant);

impl Uptime {
    /// # Create a new [`Uptime`]
    fn new() -> Self { Self(Instant::now()) }
}

impl FormatTime for Uptime {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let elapsed = self.0.elapsed();
        let s = elapsed.as_secs();
        let ms = elapsed.subsec_millis();
        write!(w, "{s:>4}.{ms:03}")
    }
}

/// # Set up logging
pub fn log() {
    let level = env::var("LOG_LEVEL").unwrap_or_else(|_| String::from("info"));
    let filter = EnvFilter::new(level);

    let debug = cfg!(debug_assertions);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_line_number(debug)
        .with_target(debug)
        .with_level(true)
        .with_timer(Uptime::new())
        .with_writer(std::io::stdout)
        .compact()
        .init();
}
