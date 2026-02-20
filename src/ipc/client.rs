//! IPC Client Library
//!
//! Client for communicating with the daemon via IPC.

use super::protocol::{Command, Message, Payload, Response};
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// IPC client
pub struct IpcClient {
    socket_path: PathBuf,
    next_id: u64,
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new(Self::default_socket_path())
    }
}

impl IpcClient {
    /// Create a new IPC client
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            next_id: 1,
        }
    }

    /// Get default socket path
    pub fn default_socket_path() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            dirs::runtime_dir()
                .or_else(dirs::cache_dir)
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("onevox")
                .join("onevox.sock")
        }

        #[cfg(target_os = "linux")]
        {
            dirs::runtime_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("onevox")
                .join("onevox.sock")
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            PathBuf::from("/tmp/onevox/onevox.sock")
        }
    }

    /// Send a command and wait for response
    pub async fn send_command(&mut self, command: Command) -> Result<Response> {
        // Connect to socket
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .context("Failed to connect to daemon. Is it running?")?;

        // Create message
        let id = self.next_id;
        self.next_id += 1;
        let message = Message::request(id, command);

        // Serialize message
        let message_bytes = bincode::serialize(&message)?;
        let len = message_bytes.len() as u32;

        // Send length + message
        stream.write_all(&len.to_le_bytes()).await?;
        stream.write_all(&message_bytes).await?;
        stream.flush().await?;

        // Read response length
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let response_len = u32::from_le_bytes(len_bytes) as usize;

        // Read response data
        let mut response_buf = vec![0u8; response_len];
        stream.read_exact(&mut response_buf).await?;

        // Deserialize response
        let response_msg: Message = bincode::deserialize(&response_buf)?;

        // Extract response payload
        match response_msg.payload {
            Payload::Response(response) => Ok(response),
            _ => Err(anyhow::anyhow!("Invalid response type")),
        }
    }

    /// Check if daemon is running
    pub async fn ping(&mut self) -> Result<bool> {
        match self.send_command(Command::Ping).await {
            Ok(Response::Pong) => Ok(true),
            Ok(_) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    /// Get daemon status
    pub async fn get_status(&mut self) -> Result<super::protocol::DaemonStatus> {
        match self.send_command(Command::GetStatus).await? {
            Response::Status(status) => Ok(status),
            Response::Error(e) => Err(anyhow::anyhow!("Error: {}", e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    /// Shutdown the daemon
    pub async fn shutdown(&mut self) -> Result<()> {
        match self.send_command(Command::Shutdown).await? {
            Response::Success => Ok(()),
            Response::Error(e) => Err(anyhow::anyhow!("Error: {}", e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    /// Get daemon configuration
    pub async fn get_config(&mut self) -> Result<String> {
        match self.send_command(Command::GetConfig).await? {
            Response::Config(config) => Ok(config),
            Response::Error(e) => Err(anyhow::anyhow!("Error: {}", e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }
}
