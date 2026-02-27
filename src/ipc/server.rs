//! IPC Server Implementation
//!
//! Unix socket server for handling daemon commands.

use super::protocol::{Command, Message, Payload, Response};
use crate::daemon::state::DaemonState as DaemonStateManager;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

/// IPC server
pub struct IpcServer {
    socket_path: PathBuf,
    listener: Option<UnixListener>,
    state: Arc<RwLock<DaemonStateManager>>,
    request_limiter: Arc<Mutex<HashMap<u32, Instant>>>,
    min_request_interval: Duration,
}

impl IpcServer {
    /// Create a new IPC server
    pub fn new(socket_path: PathBuf, state: Arc<RwLock<DaemonStateManager>>) -> Self {
        Self {
            socket_path,
            listener: None,
            state,
            request_limiter: Arc::new(Mutex::new(HashMap::new())),
            min_request_interval: Duration::from_millis(10), // Reduced from 50ms to allow faster commands
        }
    }

    /// Start the IPC server
    pub async fn start(&mut self) -> Result<()> {
        // Remove existing socket file if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        // Create parent directory if needed
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Bind to socket
        let listener = UnixListener::bind(&self.socket_path)?;
        info!("IPC server listening on {:?}", self.socket_path);

        // Set socket permissions (owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.socket_path, perms)?;
        }

        self.listener = Some(listener);
        Ok(())
    }

    /// Run the server loop
    pub async fn run(&mut self) -> Result<()> {
        let listener = self
            .listener
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Server not started"))?;

        info!("IPC server accepting connections");

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let state = Arc::clone(&self.state);
                    let request_limiter = Arc::clone(&self.request_limiter);
                    let min_request_interval = self.min_request_interval;
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(
                            stream,
                            state,
                            request_limiter,
                            min_request_interval,
                        )
                        .await
                        {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    /// Verify client credentials (Unix only)
    #[cfg(unix)]
    fn verify_client_credentials(stream: &UnixStream) -> Result<u32> {
        // Get peer credentials
        let ucred = stream
            .peer_cred()
            .map_err(|e| anyhow::anyhow!("Failed to get peer credentials: {}", e))?;

        // Get current process UID
        let current_uid = unsafe { libc::getuid() };

        // Verify same user
        if ucred.uid() != current_uid {
            warn!(
                "IPC connection attempt from different user (UID {} != {})",
                ucred.uid(),
                current_uid
            );
            return Err(anyhow::anyhow!("Unauthorized: different user"));
        }

        debug!("Client credentials verified: UID={}", ucred.uid());
        Ok(ucred.uid())
    }

    /// Verify client credentials (Windows - placeholder)
    #[cfg(not(unix))]
    fn verify_client_credentials(_stream: &UnixStream) -> Result<u32> {
        // TODO: Implement Windows credential verification
        warn!("Client credential verification not implemented on Windows");
        Ok(0)
    }

    async fn check_rate_limit(
        request_limiter: &Arc<Mutex<HashMap<u32, Instant>>>,
        client_uid: u32,
        min_request_interval: Duration,
        command: &Command,
    ) -> Result<()> {
        // Skip rate limiting for critical commands
        match command {
            Command::Shutdown | Command::Ping => return Ok(()),
            _ => {}
        }

        let now = Instant::now();
        let mut limiter = request_limiter.lock().await;

        if let Some(last_request) = limiter.get(&client_uid)
            && now.duration_since(*last_request) < min_request_interval
        {
            return Err(anyhow::anyhow!("Rate limited"));
        }

        limiter.insert(client_uid, now);
        Ok(())
    }

    /// Handle a client connection
    async fn handle_client(
        mut stream: UnixStream,
        state: Arc<RwLock<DaemonStateManager>>,
        request_limiter: Arc<Mutex<HashMap<u32, Instant>>>,
        min_request_interval: Duration,
    ) -> Result<()> {
        debug!("New IPC client connected");

        // SECURITY: Verify client credentials first
        let client_uid = Self::verify_client_credentials(&stream)?;

        // Read message length (4 bytes)
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let message_len = u32::from_le_bytes(len_bytes) as usize;

        // Sanity check message size (max 1MB)
        if message_len > 1_000_000 {
            warn!("Rejecting oversized message: {} bytes", message_len);
            return Err(anyhow::anyhow!("Message too large"));
        }

        // Read message data
        let mut message_buf = vec![0u8; message_len];
        stream.read_exact(&mut message_buf).await?;

        // Deserialize message
        let message: Message = bincode::deserialize(&message_buf)?;
        debug!("Received message: {:?}", message);

        // Check rate limit based on command type
        let response = match &message.payload {
            Payload::Request(command) => {
                // Check rate limit (skips for critical commands)
                if let Err(e) = Self::check_rate_limit(
                    &request_limiter,
                    client_uid,
                    min_request_interval,
                    command,
                )
                .await
                {
                    Response::Error(format!("Rate limited: {}", e))
                } else {
                    Self::handle_command(command.clone(), &state).await
                }
            }
            _ => Response::Error("Invalid message type".to_string()),
        };

        // Send response
        let response_msg = Message::response(message.id, response);
        let response_bytes = bincode::serialize(&response_msg)?;

        // Write response length + data
        let len = response_bytes.len() as u32;
        stream.write_all(&len.to_le_bytes()).await?;
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;

        debug!("Response sent");
        Ok(())
    }

    /// Handle a command and generate response
    async fn handle_command(command: Command, state: &Arc<RwLock<DaemonStateManager>>) -> Response {
        match command {
            Command::Ping => Response::Pong,

            Command::GetStatus => {
                let state = state.read().await;
                Response::Status(state.status())
            }

            Command::Shutdown => {
                info!("Shutdown command received");
                let mut state = state.write().await;
                state.shutdown();
                Response::Success
            }

            Command::ReloadConfig => {
                info!("Reload config command received");
                let mut state = state.write().await;
                match state.reload_config() {
                    Ok(()) => {
                        info!("Configuration reloaded successfully");
                        Response::Success
                    }
                    Err(e) => {
                        error!("Failed to reload config: {}", e);
                        Response::Error(format!("Failed to reload config: {}", e))
                    }
                }
            }

            Command::GetConfig => {
                let state = state.read().await;
                match toml::to_string_pretty(&state.config()) {
                    Ok(config_str) => Response::Config(config_str),
                    Err(e) => Response::Error(format!("Failed to serialize config: {}", e)),
                }
            }

            Command::StartDictation => {
                info!("Start dictation command received");
                let state = state.read().await;
                match state.start_dictation() {
                    Ok(()) => Response::Success,
                    Err(e) => Response::Error(format!("Failed to start dictation: {}", e)),
                }
            }

            Command::StopDictation => {
                info!("Stop dictation command received");
                let state = state.read().await;
                match state.stop_dictation() {
                    Ok(()) => Response::Success,
                    Err(e) => Response::Error(format!("Failed to stop dictation: {}", e)),
                }
            }

            Command::ListDevices => {
                // TODO: Implement device listing
                Response::List(vec!["default".to_string()])
            }

            Command::ListModels => {
                // TODO: Implement model listing
                Response::List(vec![])
            }

            Command::LoadModel { path } => {
                info!("Load model command: {}", path);
                // TODO: Implement model loading (backend auto-detected from path)
                Response::Ok(format!("Model loaded (not yet implemented): {}", path))
            }

            Command::UnloadModel => {
                info!("Unload model command received");
                // TODO: Implement model unloading
                Response::Ok("Model unloaded (not yet implemented)".to_string())
            }

            Command::GetHistory => {
                info!("Get history command received");
                let state = state.read().await;
                match state.history_manager().get_all().await {
                    Ok(entries) => Response::History(entries),
                    Err(e) => Response::Error(format!("Failed to get history: {}", e)),
                }
            }

            Command::DeleteHistoryEntry { id } => {
                info!("Delete history entry command received: {}", id);
                let state = state.read().await;
                match state.history_manager().delete_entry(id).await {
                    Ok(true) => Response::Ok(format!("Entry {} deleted", id)),
                    Ok(false) => Response::Error(format!("Entry {} not found", id)),
                    Err(e) => Response::Error(format!("Failed to delete entry: {}", e)),
                }
            }

            Command::ClearHistory => {
                info!("Clear history command received");
                let state = state.read().await;
                match state.history_manager().clear().await {
                    Ok(()) => Response::Ok("History cleared".to_string()),
                    Err(e) => Response::Error(format!("Failed to clear history: {}", e)),
                }
            }
        }
    }

    /// Stop the server and clean up
    pub fn stop(&mut self) -> Result<()> {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
            info!("IPC socket removed");
        }
        Ok(())
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
