use crate::config::LoggingConfig;
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use std::str::FromStr;
use tracing::Level;

pub fn init(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let level = Level::from_str(&config.level).unwrap_or(Level::INFO);
    let filter = EnvFilter::from_default_env()
        .add_directive(level.into());

    let registry = tracing_subscriber::registry().with(filter);

    if config.format.to_lowercase() == "json" {
        let fmt_layer = fmt::layer()
            .json()
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false);
        
        registry.with(fmt_layer).try_init()?;
    } else {
        let fmt_layer = fmt::layer()
            .with_target(false)
            .with_thread_ids(false);
        
        registry.with(fmt_layer).try_init()?;
    }

    Ok(())
}
