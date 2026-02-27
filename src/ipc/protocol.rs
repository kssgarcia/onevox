//! IPC Protocol Definitions
//!
//! Binary message protocol using bincode for efficient serialization.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// IPC message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique request ID for correlation
    pub id: u64,
    /// Message payload
    pub payload: Payload,
}

/// Message payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Payload {
    /// Request from client
    Request(Command),
    /// Response from daemon
    Response(Response),
    /// Unsolicited event from daemon
    Event(Event),
}

/// Commands that can be sent to the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Check if daemon is running
    Ping,

    /// Get daemon status
    GetStatus,

    /// Shutdown the daemon
    Shutdown,

    /// Reload configuration
    ReloadConfig,

    /// Get current configuration
    GetConfig,

    /// Start dictation mode
    StartDictation,

    /// Stop dictation mode
    StopDictation,

    /// List available audio devices
    ListDevices,

    /// List available models
    ListModels,

    /// Load a model (backend auto-detected from path)
    LoadModel { path: String },

    /// Unload current model
    UnloadModel,

    /// Get transcription history
    GetHistory,

    /// Delete a specific history entry
    DeleteHistoryEntry { id: u64 },

    /// Clear all history
    ClearHistory,
}

/// Responses from the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Operation succeeded
    Success,

    /// Operation succeeded with data
    Ok(String),

    /// Operation failed
    Error(String),

    /// Daemon status
    Status(DaemonStatus),

    /// Configuration data
    Config(String), // TOML-serialized config

    /// List of items
    List(Vec<String>),

    /// Pong response
    Pong,

    /// History entries
    History(Vec<crate::history::HistoryEntry>),
}

/// Daemon status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// Daemon version
    pub version: String,

    /// Process ID
    pub pid: u32,

    /// Uptime in seconds
    pub uptime_secs: u64,

    /// Current state
    pub state: DaemonState,

    /// Is model loaded
    pub model_loaded: bool,

    /// Current model name (if loaded)
    pub model_name: Option<String>,

    /// Is currently dictating
    pub is_dictating: bool,

    /// Memory usage in bytes
    pub memory_usage_bytes: u64,

    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: f32,
}

/// Daemon operational state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DaemonState {
    /// Daemon is starting up
    Starting,

    /// Daemon is idle and ready
    Idle,

    /// Daemon is actively processing audio
    Active,

    /// Daemon is shutting down
    ShuttingDown,

    /// Daemon encountered an error
    Error,
}

/// Events emitted by the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Daemon started successfully
    Started,

    /// Daemon is shutting down
    ShuttingDown,

    /// Model loaded
    ModelLoaded { name: String },

    /// Model unloaded
    ModelUnloaded,

    /// Transcription completed
    TranscriptionComplete { text: String, duration_ms: u64 },

    /// Error occurred
    Error { message: String },

    /// Log message
    Log {
        level: String,
        message: String,
        timestamp: SystemTime,
    },
}

impl Message {
    /// Create a new request message
    pub fn request(id: u64, command: Command) -> Self {
        Self {
            id,
            payload: Payload::Request(command),
        }
    }

    /// Create a new response message
    pub fn response(id: u64, response: Response) -> Self {
        Self {
            id,
            payload: Payload::Response(response),
        }
    }

    /// Create a new event message
    pub fn event(id: u64, event: Event) -> Self {
        Self {
            id,
            payload: Payload::Event(event),
        }
    }
}

impl DaemonStatus {
    /// Create a new status with defaults
    pub fn new(pid: u32, uptime_secs: u64) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            pid,
            uptime_secs,
            state: DaemonState::Starting,
            model_loaded: false,
            model_name: None,
            is_dictating: false,
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
        }
    }
}

impl std::fmt::Display for DaemonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonState::Starting => write!(f, "Starting"),
            DaemonState::Idle => write!(f, "Idle"),
            DaemonState::Active => write!(f, "Active"),
            DaemonState::ShuttingDown => write!(f, "Shutting Down"),
            DaemonState::Error => write!(f, "Error"),
        }
    }
}
