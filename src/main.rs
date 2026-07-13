use clap::Parser;
use socks5_bridge::cli::{Cli, Commands};
use socks5_bridge::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config, foreground, validate_only } => {
            let app_config = AppConfig::load_from_file(&config)?;
            if validate_only {
                println!("Configuration is valid.");
                return Ok(());
            }

            socks5_bridge::logging::init(&app_config.logging)?;
            
            if !foreground {
                tracing::warn!("Daemon mode not implemented yet, running in foreground");
            }
            
            tracing::info!(
                event = "startup",
                listener = format!("{}:{}", app_config.listener.host, app_config.listener.port),
                admin = format!("{}:{}", app_config.health.admin_host, app_config.health.admin_port),
                upstream_host = app_config.upstream.host,
                upstream_port = app_config.upstream.port
            );

            socks5_bridge::app::App::run(app_config).await?;
        }
        Commands::Check { config } => {
            if let Err(e) = AppConfig::load_from_file(&config) {
                eprintln!("Configuration is invalid: {}", e);
                std::process::exit(1);
            }
            println!("Configuration is valid.");
        }
        Commands::TestUpstream { config: _ } => {
            println!("test-upstream not implemented yet.");
        }
        Commands::Status { admin_port: _ } => {
            println!("status not implemented yet.");
        }
        Commands::Stop { admin_port: _ } => {
            println!("stop not implemented yet.");
        }
        Commands::PrintChromeArgs { config } => {
            let app_config = AppConfig::load_from_file(&config)?;
            println!(
                "/Applications/Google\\ Chrome.app/Contents/MacOS/Google\\ Chrome \\\n  --proxy-server=\"http://{}:{}\" --disable-quic\n# Note: Chrome is pointed to the local bridge to handle authenticated SOCKS5 upstream.",
                app_config.listener.host, app_config.listener.port
            );
        }
    }

    Ok(())
}
