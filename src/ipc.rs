//! Inter-process communication
//!
//! IPC server and protocol for controlling the daemon.

pub mod client;
pub mod protocol;
pub mod server;

// Re-export commonly used types
pub use client::IpcClient;
pub use protocol::{Command, DaemonStatus, Message, Payload, Response};
pub use server::IpcServer;
