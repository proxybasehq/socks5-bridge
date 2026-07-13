use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "socks5-bridge")]
#[command(about = "SOCKS5 Bridge for Chrome on macOS", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Starts the service
    Start {
        /// Path to the configuration file (TOML format)
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,

        /// Run in foreground
        #[arg(long)]
        foreground: bool,

        /// Validate configuration only and exit
        #[arg(long)]
        validate_only: bool,
    },
    
    /// Validates config without starting network listeners
    Check {
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
    },

    /// Tests only the upstream SOCKS5 connection and auth path
    TestUpstream {
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
    },

    /// Queries admin API for state
    Status {
        #[arg(short = 'p', long, default_value = "8898")]
        admin_port: u16,
    },

    /// Gracefully shuts down a running daemon through admin API
    Stop {
        #[arg(short = 'p', long, default_value = "8898")]
        admin_port: u16,
    },

    /// Prints launch arguments for Chrome
    PrintChromeArgs {
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
    },
}
