// Vox - Local Speech-to-Text Daemon
// Main binary entry point

use clap::{Parser, Subcommand};
use vox::{Config, Result};
use toml;

#[derive(Parser)]
#[command(name = "vox")]
#[command(about = "Ultra-fast local speech-to-text daemon", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon
    Daemon {
        /// Run in development mode
        #[arg(long)]
        dev: bool,
    },

    /// Check daemon status
    Status,

    /// Configure vox
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Open TUI monitor
    #[cfg(feature = "tui")]
    Tui,

    /// List audio devices
    Devices {
        #[command(subcommand)]
        action: DeviceAction,
    },

    /// Manage models
    Models {
        #[command(subcommand)]
        action: ModelAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },

    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
}

#[derive(Subcommand)]
enum DeviceAction {
    /// List available audio devices
    List,
}

#[derive(Subcommand)]
enum ModelAction {
    /// List available models
    List,

    /// Download a model
    Download {
        /// Model name (e.g., "tiny.en", "base.en")
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon { dev: _ } => {
            tracing::info!("Starting vox daemon...");
            // TODO: Implement daemon startup
            println!("🎙️  Vox daemon starting...");
            println!("⚠️  Not yet implemented - this is a placeholder");
            Ok(())
        }

        Commands::Status => {
            println!("📊 Vox daemon status:");
            println!("⚠️  Not yet implemented - this is a placeholder");
            Ok(())
        }

        Commands::Config { action } => match action {
            ConfigAction::Show => {
                let config = Config::load_default()?;
                let config_str = toml::to_string_pretty(&config)
                    .map_err(|e| vox::Error::Config(format!("Failed to serialize: {}", e)))?;
                println!("📝 Current configuration:\n");
                println!("{}", config_str);
                println!("\nConfig file: {:?}", Config::default_path());
                Ok(())
            }
            ConfigAction::Set { key, value } => {
                println!("Setting {key} = {value}");
                println!("⚠️  Not yet implemented - this is a placeholder");
                Ok(())
            }
            ConfigAction::Get { key } => {
                println!("Getting {key}");
                println!("⚠️  Not yet implemented - this is a placeholder");
                Ok(())
            }
        },

        #[cfg(feature = "tui")]
        Commands::Tui => {
            println!("🖥️  Opening TUI monitor...");
            println!("⚠️  Not yet implemented - this is a placeholder");
            Ok(())
        }

        Commands::Devices { action } => match action {
            DeviceAction::List => {
                println!("🎤 Available audio devices:");
                println!("⚠️  Not yet implemented - this is a placeholder");
                Ok(())
            }
        },

        Commands::Models { action } => match action {
            ModelAction::List => {
                println!("🤖 Available models:");
                println!("⚠️  Not yet implemented - this is a placeholder");
                Ok(())
            }
            ModelAction::Download { name } => {
                println!("📥 Downloading model: {name}");
                println!("⚠️  Not yet implemented - this is a placeholder");
                Ok(())
            }
        },
    }
}
