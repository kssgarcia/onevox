// Onevox - Local Speech-to-Text Daemon
// Main binary entry point

use clap::{Parser, Subcommand};
use onevox::{Config, Result};

#[derive(Parser)]
#[command(name = "onevox")]
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

        /// Run in foreground (don't daemonize)
        #[arg(long)]
        foreground: bool,
    },

    /// Stop the daemon
    Stop,

    /// Check daemon status
    Status,

    /// Configure onevox
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Open TUI monitor
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

    /// View and manage transcription history
    History {
        #[command(subcommand)]
        action: HistoryAction,
    },

    /// Test audio capture (dev tool)
    TestAudio {
        /// Duration in seconds
        #[arg(short, long, default_value = "3")]
        duration: u64,
    },

    /// Test VAD (dev tool)
    TestVad {
        /// Duration in seconds
        #[arg(short, long, default_value = "10")]
        duration: u64,
    },

    /// Test full transcription pipeline (dev tool)
    TestTranscribe {
        /// Duration in seconds
        #[arg(short, long, default_value = "10")]
        duration: u64,
    },

    /// Test hotkey detection (dev tool)
    TestHotkey {
        /// Hotkey combination (e.g., "Cmd+Shift+Space")
        #[arg(short, long, default_value = "Cmd+Shift+Space")]
        hotkey: String,
    },

    /// Start dictation (for Wayland/manual triggering)
    StartDictation,

    /// Stop dictation (for Wayland/manual triggering)
    StopDictation,

    /// Internal overlay indicator process
    #[command(hide = true)]
    Indicator {
        /// Indicator mode: recording or processing
        #[arg(long)]
        mode: String,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Initialize default configuration file
    Init,

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
    /// List all available models from registry
    List,

    /// Show downloaded models
    Downloaded,

    /// Download a model
    Download {
        /// Model ID (e.g., "whisper-tiny.en", "whisper-base.en")
        model_id: String,
    },

    /// Remove a downloaded model
    Remove {
        /// Model ID to remove
        model_id: String,
    },

    /// Show model information
    Info {
        /// Model ID
        model_id: String,
    },
}

#[derive(Subcommand)]
enum HistoryAction {
    /// List all transcription history
    List {
        /// Number of recent entries to show (0 = all)
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Delete a specific history entry
    Delete {
        /// Entry ID to delete
        id: u64,
    },

    /// Clear all history
    Clear {
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Export history to a file
    Export {
        /// Output file path
        #[arg(short, long, default_value = "transcription-history.txt")]
        output: String,
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
        Commands::Daemon { dev, foreground } => {
            tracing::info!("Starting onevox daemon...");

            // Load configuration
            let config = Config::load_default()?;

            if dev {
                tracing::info!("Running in development mode");
            }

            if !foreground {
                println!("🎙️  Starting Onevox daemon in background...");
                println!("    Use 'onevox status' to check status");
                println!("    Use 'onevox stop' to stop the daemon");
            }

            // Create and start daemon
            let mut daemon = onevox::Daemon::new_async(config).await;
            daemon.start().await?;

            Ok(())
        }

        Commands::Stop => {
            println!("🛑 Stopping Onevox daemon...");
            match onevox::Daemon::stop().await {
                Ok(_) => {
                    println!("✅ Daemon stopped successfully");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to stop daemon: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Status => match onevox::Daemon::status().await {
            Ok(status) => {
                println!("📊 Onevox Daemon Status\n");
                println!("  Version:     {}", status.version);
                println!("  PID:         {}", status.pid);
                println!("  State:       {}", status.state);
                println!("  Uptime:      {}s", status.uptime_secs);
                println!(
                    "  Model:       {}",
                    status.model_name.unwrap_or_else(|| "None".to_string())
                );
                println!(
                    "  Dictating:   {}",
                    if status.is_dictating { "Yes" } else { "No" }
                );
                println!(
                    "  Memory:      {} MB",
                    status.memory_usage_bytes / 1_000_000
                );
                println!("  CPU:         {:.1}%", status.cpu_usage_percent);
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to get daemon status: {}", e);
                eprintln!("💡 Is the daemon running? Try: onevox daemon --foreground");
                std::process::exit(1);
            }
        },

        Commands::Config { action } => match action {
            ConfigAction::Show => {
                let config = Config::load_default()?;
                let config_str = toml::to_string_pretty(&config)
                    .map_err(|e| onevox::Error::Config(format!("Failed to serialize: {}", e)))?;
                println!("📝 Current configuration:\n");
                println!("{}", config_str);
                println!("\nConfig file: {:?}", Config::default_path());
                Ok(())
            }
            ConfigAction::Init => {
                let config_path = Config::default_path();

                if config_path.exists() {
                    println!("⚠️  Config file already exists at: {:?}", config_path);
                    println!("Delete it first if you want to reinitialize.");
                    return Ok(());
                }

                let default_config = Config::default();
                default_config.save_default()?;

                println!("✅ Created default config at: {:?}", config_path);
                println!("\n📝 Default hotkey: Cmd+Shift+0");
                println!("Edit the file to customize settings.");
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

        Commands::Tui => onevox::tui::launch(),

        Commands::Devices { action } => match action {
            DeviceAction::List => {
                println!("🎤 Available audio input devices:\n");
                let audio_engine = onevox::audio::AudioEngine::new();
                match audio_engine.list_devices() {
                    Ok(devices) => {
                        if devices.is_empty() {
                            println!("  No audio input devices found");
                        } else {
                            for (i, device) in devices.iter().enumerate() {
                                println!("  {}. {}", i + 1, device);
                            }
                        }
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to list devices: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },

        Commands::Models { action } => match action {
            ModelAction::List => {
                use onevox::models::ModelRegistry;

                println!("🤖 Available Whisper Models\n");

                let registry = ModelRegistry::new();
                let models = registry.list_models();

                for model in models {
                    println!("📦 {}", model.name);
                    println!("   ID: {}", model.id);
                    println!(
                        "   Size: {:.1} MB",
                        model.size_bytes as f64 / 1024.0 / 1024.0
                    );
                    println!("   Speed: {}x real-time", model.speed_factor);
                    println!("   Memory: {} MB", model.memory_mb);
                    println!("   {}", model.description);
                    println!();
                }

                println!("💡 Recommended: whisper-base.en (good balance of speed and accuracy)");
                println!("💡 Download with: onevox models download <model-id>");

                Ok(())
            }

            ModelAction::Downloaded => {
                use onevox::models::ModelDownloader;

                println!("📂 Downloaded Models\n");

                let downloader =
                    ModelDownloader::new().map_err(|e| onevox::Error::Other(e.to_string()))?;
                let downloaded = downloader
                    .list_downloaded()
                    .await
                    .map_err(|e| onevox::Error::Other(e.to_string()))?;

                if downloaded.is_empty() {
                    println!("No models downloaded yet.");
                    println!("💡 Download a model with: onevox models download <model-id>");
                } else {
                    for model_id in downloaded {
                        let size = downloader
                            .model_size(&model_id)
                            .await
                            .map_err(|e| onevox::Error::Other(e.to_string()))?;
                        println!("✅ {} ({:.1} MB)", model_id, size as f64 / 1024.0 / 1024.0);
                    }
                }

                Ok(())
            }

            ModelAction::Download { model_id } => {
                use onevox::models::{ModelDownloader, ModelRegistry};

                println!("📥 Downloading model: {}\n", model_id);

                let registry = ModelRegistry::new();
                let metadata = registry.get_model(&model_id).ok_or_else(|| {
                    onevox::Error::Config(format!("Model not found: {}", model_id))
                })?;

                let downloader =
                    ModelDownloader::new().map_err(|e| onevox::Error::Other(e.to_string()))?;

                // Check if already downloaded
                if downloader.is_downloaded(metadata).await {
                    println!("✅ Model already downloaded!");
                    println!("💡 Location: {:?}", downloader.model_dir(&model_id));
                    return Ok(());
                }

                println!("Model: {}", metadata.name);
                println!(
                    "Size: {:.1} MB",
                    metadata.size_bytes as f64 / 1024.0 / 1024.0
                );
                println!("Files: {} files", metadata.files.len());
                println!();

                // Download
                let model_dir = downloader
                    .download(metadata)
                    .await
                    .map_err(|e| onevox::Error::Other(e.to_string()))?;

                println!("\n✅ Model downloaded successfully!");
                println!("📂 Location: {:?}", model_dir);
                println!("💡 Update your config to use this model");

                Ok(())
            }

            ModelAction::Remove { model_id } => {
                use onevox::models::ModelDownloader;

                println!("🗑️  Removing model: {}", model_id);

                let downloader =
                    ModelDownloader::new().map_err(|e| onevox::Error::Other(e.to_string()))?;
                downloader
                    .remove(&model_id)
                    .await
                    .map_err(|e| onevox::Error::Other(e.to_string()))?;

                println!("✅ Model removed successfully");

                Ok(())
            }

            ModelAction::Info { model_id } => {
                use onevox::models::{ModelDownloader, ModelRegistry};

                let registry = ModelRegistry::new();
                let metadata = registry.get_model(&model_id).ok_or_else(|| {
                    onevox::Error::Config(format!("Model not found: {}", model_id))
                })?;

                println!("📦 {}\n", metadata.name);
                println!("ID:          {}", metadata.id);
                println!(
                    "Size:        {:.1} MB",
                    metadata.size_bytes as f64 / 1024.0 / 1024.0
                );
                println!("Speed:       {}x real-time", metadata.speed_factor);
                println!("Memory:      {} MB RAM required", metadata.memory_mb);
                println!("Repository:  {}", metadata.hf_repo);
                println!("Files:       {}", metadata.files.len());
                println!("\nDescription:");
                println!("  {}", metadata.description);

                // Check if downloaded
                let downloader =
                    ModelDownloader::new().map_err(|e| onevox::Error::Other(e.to_string()))?;
                if downloader.is_downloaded(metadata).await {
                    let size = downloader
                        .model_size(&model_id)
                        .await
                        .map_err(|e| onevox::Error::Other(e.to_string()))?;
                    println!(
                        "\n✅ Downloaded ({:.1} MB on disk)",
                        size as f64 / 1024.0 / 1024.0
                    );
                    println!("📂 {:?}", downloader.model_dir(&model_id));
                } else {
                    println!("\n❌ Not downloaded");
                    println!("💡 Download with: onevox models download {}", model_id);
                }

                Ok(())
            }
        },

        Commands::History { action } => match action {
            HistoryAction::List { limit } => {
                let mut client = onevox::ipc::IpcClient::default();

                match client.get_history().await {
                    Ok(mut entries) => {
                        if entries.is_empty() {
                            println!("📝 No transcription history yet");
                            println!("💡 Start dictating to build your history!");
                            return Ok(());
                        }

                        // Sort by timestamp, newest first
                        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                        // Apply limit
                        let to_show = if limit == 0 || limit >= entries.len() {
                            entries.len()
                        } else {
                            limit
                        };

                        println!("📝 Transcription History ({} entries)\n", entries.len());
                        println!("Showing {} most recent:\n", to_show);

                        for (i, entry) in entries.iter().take(to_show).enumerate() {
                            // Format timestamp
                            let datetime =
                                chrono::DateTime::from_timestamp(entry.timestamp as i64, 0)
                                    .or_else(|| chrono::DateTime::from_timestamp(0, 0))
                                    .unwrap_or(chrono::DateTime::UNIX_EPOCH);
                            let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S");

                            println!("─────────────────────────────────────────");
                            println!("#{} [ID: {}]", i + 1, entry.id);
                            println!("📅 {}", formatted_time);
                            println!("🤖 Model: {}", entry.model);
                            println!("⏱️  Duration: {}ms", entry.duration_ms);
                            if let Some(conf) = entry.confidence {
                                println!("📊 Confidence: {:.1}%", conf * 100.0);
                            }
                            println!("\n💬 \"{}\"", entry.text);
                            println!();
                        }

                        if entries.len() > to_show {
                            println!("... and {} more entries", entries.len() - to_show);
                            println!("💡 Use --limit 0 to show all entries");
                        }

                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to get history: {}", e);
                        eprintln!("💡 Is the daemon running? Try: onevox daemon --foreground");
                        std::process::exit(1);
                    }
                }
            }

            HistoryAction::Delete { id } => {
                let mut client = onevox::ipc::IpcClient::default();

                match client.delete_history_entry(id).await {
                    Ok(_) => {
                        println!("✅ Deleted history entry #{}", id);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to delete entry: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            HistoryAction::Clear { yes } => {
                if !yes {
                    println!("⚠️  This will delete ALL transcription history.");
                    print!("Are you sure? (y/N): ");
                    use std::io::{self, Write};
                    if let Err(e) = io::stdout().flush() {
                        eprintln!("Warning: Failed to flush stdout: {}", e);
                    }

                    let mut input = String::new();
                    if let Err(e) = io::stdin().read_line(&mut input) {
                        eprintln!("❌ Failed to read input: {}", e);
                        std::process::exit(1);
                    }

                    if !input.trim().eq_ignore_ascii_case("y") {
                        println!("Cancelled.");
                        return Ok(());
                    }
                }

                let mut client = onevox::ipc::IpcClient::default();

                match client.clear_history().await {
                    Ok(_) => {
                        println!("✅ All history cleared");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to clear history: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            HistoryAction::Export { output } => {
                use std::fs::File;
                use std::io::Write;

                let mut client = onevox::ipc::IpcClient::default();

                match client.get_history().await {
                    Ok(mut entries) => {
                        if entries.is_empty() {
                            println!("📝 No history to export");
                            return Ok(());
                        }

                        // Sort by timestamp
                        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

                        // Write to file
                        let mut file = File::create(&output).map_err(|e| {
                            onevox::Error::Other(format!("Failed to create file: {}", e))
                        })?;

                        writeln!(file, "Onevox Transcription History")
                            .map_err(|e| onevox::Error::Other(format!("Failed to write: {}", e)))?;
                        writeln!(
                            file,
                            "Generated: {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                        )
                        .map_err(|e| onevox::Error::Other(format!("Failed to write: {}", e)))?;
                        writeln!(file, "Total entries: {}\n", entries.len())
                            .map_err(|e| onevox::Error::Other(format!("Failed to write: {}", e)))?;
                        writeln!(
                            file,
                            "============================================================\n"
                        )
                        .map_err(|e| onevox::Error::Other(format!("Failed to write: {}", e)))?;

                        let entry_count = entries.len();
                        for entry in entries {
                            let datetime =
                                chrono::DateTime::from_timestamp(entry.timestamp as i64, 0)
                                    .or_else(|| chrono::DateTime::from_timestamp(0, 0))
                                    .unwrap_or(chrono::DateTime::UNIX_EPOCH);
                            let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S");

                            writeln!(
                                file,
                                "[{}] ({}ms) {}",
                                formatted_time, entry.duration_ms, entry.model
                            )
                            .map_err(|e| onevox::Error::Other(format!("Failed to write: {}", e)))?;
                            writeln!(file, "{}\n", entry.text).map_err(|e| {
                                onevox::Error::Other(format!("Failed to write: {}", e))
                            })?;
                        }

                        println!("✅ Exported {} entries to {}", entry_count, output);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to get history: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },

        Commands::TestAudio { duration } => {
            println!("🎤 Testing audio capture for {} seconds...", duration);
            println!("Speak into your microphone!\n");

            let config = onevox::audio::CaptureConfig::default();
            let mut engine = onevox::audio::AudioEngine::new();

            let mut chunk_rx = engine.start_capture(config)?;

            let start = std::time::Instant::now();
            let mut chunk_count = 0;
            let mut total_samples = 0;

            while start.elapsed().as_secs() < duration {
                if let Ok(chunk) = chunk_rx.try_recv() {
                    chunk_count += 1;
                    total_samples += chunk.len();
                    println!(
                        "  Chunk {}: {} samples, {:.1}ms",
                        chunk_count,
                        chunk.len(),
                        chunk.duration_ms()
                    );
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            engine.stop_capture()?;

            println!("\n✅ Capture test complete!");
            println!("  Total chunks: {}", chunk_count);
            println!("  Total samples: {}", total_samples);
            println!(
                "  Average samples/chunk: {}",
                if chunk_count > 0 {
                    total_samples / chunk_count
                } else {
                    0
                }
            );

            Ok(())
        }

        Commands::TestVad { duration } => {
            println!("🎤 Testing VAD for {} seconds...", duration);
            println!("Speak into your microphone to see speech detection!\n");

            // Load config
            let config = Config::load_default()?;

            // Create audio engine
            let audio_config = onevox::audio::CaptureConfig::default();
            let mut engine = onevox::audio::AudioEngine::new();
            let mut chunk_rx = engine.start_capture(audio_config)?;

            // Create VAD processor
            let energy_config = config.vad.to_energy_vad_config();
            let processor_config = config.vad.to_processor_config();
            let detector = Box::new(onevox::vad::EnergyVad::new(energy_config));
            let mut vad_processor = onevox::vad::VadProcessor::new(processor_config, detector);

            println!("VAD Configuration:");
            println!("  Detector: {}", vad_processor.detector_name());
            println!("  Threshold: {}", config.vad.threshold);
            println!("  Pre-roll: {}ms", config.vad.pre_roll_ms);
            println!("  Post-roll: {}ms", config.vad.post_roll_ms);
            println!("  Adaptive: {}\n", config.vad.adaptive);

            let start = std::time::Instant::now();
            let mut speech_segments = 0;
            let mut current_state = "🔇 Silence";

            while start.elapsed().as_secs() < duration {
                if let Ok(chunk) = chunk_rx.try_recv() {
                    match vad_processor.process(chunk)? {
                        Some(segment) => {
                            speech_segments += 1;
                            println!(
                                "🎙️  Speech segment #{}: {} chunks, {}ms duration",
                                speech_segments,
                                segment.len(),
                                segment.duration_ms
                            );
                            current_state = "🔇 Silence";
                        }
                        None => {
                            let new_state = if vad_processor.is_in_speech() {
                                "🔴 Speech"
                            } else {
                                "🔇 Silence"
                            };
                            if new_state != current_state {
                                println!("{}", new_state);
                                current_state = new_state;
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            engine.stop_capture()?;

            println!("\n✅ VAD test complete!");
            println!("  Total speech segments: {}", speech_segments);

            Ok(())
        }

        Commands::TestTranscribe { duration } => {
            println!(
                "🎤 Testing full transcription pipeline for {} seconds...",
                duration
            );
            println!("Speak into your microphone to see real-time transcription!\n");

            // Load config
            let config = Config::load_default()?;

            // Create and load model
            use onevox::models::{MockModel, ModelConfig, ModelRuntime};
            let mut model = MockModel::new();
            let model_config = ModelConfig::default();
            model.load(model_config)?;

            println!("Model: {}", model.name());
            println!("Model info: {:?}\n", model.info());

            // Create audio engine
            let audio_config = onevox::audio::CaptureConfig::default();
            let mut engine = onevox::audio::AudioEngine::new();
            let mut chunk_rx = engine.start_capture(audio_config)?;

            // Create VAD processor
            let energy_config = config.vad.to_energy_vad_config();
            let processor_config = config.vad.to_processor_config();
            let detector = Box::new(onevox::vad::EnergyVad::new(energy_config));
            let mut vad_processor = onevox::vad::VadProcessor::new(processor_config, detector);

            println!("VAD Configuration:");
            println!("  Detector: {}", vad_processor.detector_name());
            println!("  Threshold: {}", config.vad.threshold);
            println!("  Pre-roll: {}ms", config.vad.pre_roll_ms);
            println!("  Post-roll: {}ms\n", config.vad.post_roll_ms);

            let start = std::time::Instant::now();
            let mut transcription_count = 0;
            let mut current_state = "🔇 Silence";

            while start.elapsed().as_secs() < duration {
                if let Ok(chunk) = chunk_rx.try_recv() {
                    match vad_processor.process(chunk)? {
                        Some(mut segment) => {
                            transcription_count += 1;
                            println!("\n🎙️  Speech segment #{}:", transcription_count);
                            println!("  Duration: {}ms", segment.duration_ms);
                            println!("  Chunks: {}", segment.len());

                            // Transcribe the segment
                            let transcription = model.transcribe_segment(&mut segment)?;
                            println!("  📝 Transcription: \"{}\"", transcription.text);
                            println!(
                                "  ⏱️  Processing time: {}ms",
                                transcription.processing_time_ms
                            );
                            if let Some(conf) = transcription.confidence {
                                println!("  📊 Confidence: {:.2}%", conf * 100.0);
                            }

                            current_state = "🔇 Silence";
                        }
                        None => {
                            let new_state = if vad_processor.is_in_speech() {
                                "🔴 Speech"
                            } else {
                                "🔇 Silence"
                            };
                            if new_state != current_state {
                                println!("{}", new_state);
                                current_state = new_state;
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            engine.stop_capture()?;
            model.unload();

            println!("\n✅ Transcription test complete!");
            println!("  Total transcriptions: {}", transcription_count);

            Ok(())
        }

        Commands::StartDictation => {
            println!("🎤 Starting dictation...");
            let mut client = onevox::ipc::IpcClient::default();
            match client.start_dictation().await {
                Ok(_) => {
                    println!("✅ Dictation started");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to start dictation: {}", e);
                    eprintln!("💡 Is the daemon running? Try: onevox daemon --foreground");
                    std::process::exit(1);
                }
            }
        }

        Commands::StopDictation => {
            println!("🛑 Stopping dictation...");
            let mut client = onevox::ipc::IpcClient::default();
            match client.stop_dictation().await {
                Ok(_) => {
                    println!("✅ Dictation stopped");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to stop dictation: {}", e);
                    eprintln!("💡 Is the daemon running? Try: onevox daemon --foreground");
                    std::process::exit(1);
                }
            }
        }

        Commands::Indicator { mode } => {
            let parsed = onevox::indicator::IndicatorMode::from_cli(&mode).ok_or_else(|| {
                onevox::Error::Config(format!(
                    "Invalid indicator mode '{}', expected 'recording' or 'processing'",
                    mode
                ))
            })?;
            onevox::indicator::run_indicator(parsed)
        }

        Commands::TestHotkey { hotkey } => {
            println!("🎹 Testing hotkey detection...");
            println!("Hotkey: {}", hotkey);
            println!("\nPress the hotkey combination to test.");
            println!("Press Ctrl+C to exit.\n");

            // Parse hotkey config
            use onevox::platform::{HotkeyConfig, HotkeyManager};

            let hotkey_config = HotkeyConfig::from_string(&hotkey)?;
            println!("Parsed config: {:?}\n", hotkey_config);

            // Create hotkey manager
            let mut manager = HotkeyManager::new()?;

            // Register hotkey
            let mut event_rx = manager.register(hotkey_config)?;
            println!("✅ Hotkey registered successfully");

            // Start listener
            manager.start_listener()?;
            println!("✅ Listener started");
            println!("\n👂 Waiting for hotkey events...");
            println!("\n⚠️  If nothing happens when you press the hotkey:");
            println!("   You need to grant 'Input Monitoring' permission!");
            println!("   Go to: System Settings → Privacy & Security → Input Monitoring");
            println!("   Add your Terminal app and toggle it ON\n");

            let start = std::time::Instant::now();
            let mut event_count = 0;

            // Listen for events for 30 seconds, or until user quits
            loop {
                if let Ok(event) = event_rx.try_recv() {
                    event_count += 1;
                    match event {
                        onevox::platform::HotkeyEvent::Pressed => {
                            println!("🟢 PRESSED  - Hotkey detected! (event #{})", event_count);
                        }
                        onevox::platform::HotkeyEvent::Released => {
                            println!("🔴 RELEASED - Hotkey released! (event #{})", event_count);
                        }
                    }
                }

                // Show a reminder every 10 seconds if no events received
                if event_count == 0
                    && start.elapsed().as_secs().is_multiple_of(10)
                    && start.elapsed().as_secs() > 0
                {
                    println!(
                        "💡 Still waiting... Make sure you've granted Input Monitoring permission!"
                    );
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}
