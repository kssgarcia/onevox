//! Health Check System
//!
//! Monitors daemon health and provides diagnostics.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{error, warn};

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Overall health status
    pub status: HealthStatus,
    /// Individual component checks
    pub checks: Vec<ComponentCheck>,
    /// Timestamp of check
    pub timestamp: u64,
    /// Total check duration
    pub duration_ms: u64,
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Some non-critical issues
    Degraded,
    /// Critical issues present
    Unhealthy,
}

/// Individual component check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCheck {
    /// Component name
    pub name: String,
    /// Check status
    pub status: HealthStatus,
    /// Optional message
    pub message: Option<String>,
    /// Check duration in milliseconds
    pub duration_ms: u64,
}

impl ComponentCheck {
    pub fn healthy(name: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
            duration_ms,
        }
    }

    pub fn degraded(name: impl Into<String>, message: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            duration_ms,
        }
    }

    pub fn unhealthy(
        name: impl Into<String>,
        message: impl Into<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            duration_ms,
        }
    }
}

/// Health checker
pub struct HealthChecker {
    /// Memory threshold in bytes (warning)
    memory_warning_threshold: u64,
    /// Memory threshold in bytes (critical)
    memory_critical_threshold: u64,
    /// CPU threshold percentage (warning)
    cpu_warning_threshold: f32,
    /// CPU threshold percentage (critical)
    cpu_critical_threshold: f32,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self {
            memory_warning_threshold: 500 * 1024 * 1024,   // 500 MB
            memory_critical_threshold: 1024 * 1024 * 1024, // 1 GB
            cpu_warning_threshold: 50.0,                   // 50%
            cpu_critical_threshold: 80.0,                  // 80%
        }
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Perform health check
    pub async fn check(&self, daemon_state: &crate::daemon::DaemonState) -> HealthCheck {
        let start = Instant::now();
        let mut checks = Vec::new();

        // Check memory usage
        checks.push(self.check_memory(daemon_state).await);

        // Check CPU usage
        checks.push(self.check_cpu(daemon_state).await);

        // Check IPC socket
        checks.push(self.check_ipc().await);

        // Check model status
        checks.push(self.check_model(daemon_state).await);

        // Check audio system
        checks.push(self.check_audio().await);

        // Determine overall status
        let status = if checks.iter().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else if checks.iter().any(|c| c.status == HealthStatus::Degraded) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        HealthCheck {
            status,
            checks,
            timestamp,
            duration_ms,
        }
    }

    async fn check_memory(&self, daemon_state: &crate::daemon::DaemonState) -> ComponentCheck {
        let start = Instant::now();
        let status = daemon_state.status();
        let memory_bytes = status.memory_usage_bytes;
        let duration_ms = start.elapsed().as_millis() as u64;

        if memory_bytes >= self.memory_critical_threshold {
            ComponentCheck::unhealthy(
                "memory",
                format!("Memory usage critical: {} MB", memory_bytes / 1024 / 1024),
                duration_ms,
            )
        } else if memory_bytes >= self.memory_warning_threshold {
            ComponentCheck::degraded(
                "memory",
                format!("Memory usage high: {} MB", memory_bytes / 1024 / 1024),
                duration_ms,
            )
        } else {
            ComponentCheck::healthy("memory", duration_ms)
        }
    }

    async fn check_cpu(&self, daemon_state: &crate::daemon::DaemonState) -> ComponentCheck {
        let start = Instant::now();
        let status = daemon_state.status();
        let cpu_percent = status.cpu_usage_percent;
        let duration_ms = start.elapsed().as_millis() as u64;

        if cpu_percent >= self.cpu_critical_threshold {
            ComponentCheck::unhealthy(
                "cpu",
                format!("CPU usage critical: {:.1}%", cpu_percent),
                duration_ms,
            )
        } else if cpu_percent >= self.cpu_warning_threshold {
            ComponentCheck::degraded(
                "cpu",
                format!("CPU usage high: {:.1}%", cpu_percent),
                duration_ms,
            )
        } else {
            ComponentCheck::healthy("cpu", duration_ms)
        }
    }

    async fn check_ipc(&self) -> ComponentCheck {
        let start = Instant::now();

        match crate::platform::ipc_socket_path() {
            Ok(socket_path) => {
                if socket_path.exists() {
                    ComponentCheck::healthy("ipc", start.elapsed().as_millis() as u64)
                } else {
                    ComponentCheck::unhealthy(
                        "ipc",
                        "IPC socket not found",
                        start.elapsed().as_millis() as u64,
                    )
                }
            }
            Err(e) => ComponentCheck::unhealthy(
                "ipc",
                format!("Failed to get IPC socket path: {}", e),
                start.elapsed().as_millis() as u64,
            ),
        }
    }

    async fn check_model(&self, daemon_state: &crate::daemon::DaemonState) -> ComponentCheck {
        let start = Instant::now();
        let status = daemon_state.status();
        let duration_ms = start.elapsed().as_millis() as u64;

        if status.model_loaded {
            ComponentCheck::healthy("model", duration_ms)
        } else {
            ComponentCheck::degraded("model", "No model loaded", duration_ms)
        }
    }

    async fn check_audio(&self) -> ComponentCheck {
        let start = Instant::now();

        // Try to list audio devices
        let audio_engine = crate::audio::AudioEngine::new();
        match audio_engine.list_devices() {
            Ok(devices) if !devices.is_empty() => {
                ComponentCheck::healthy("audio", start.elapsed().as_millis() as u64)
            }
            Ok(_) => ComponentCheck::unhealthy(
                "audio",
                "No audio input devices found",
                start.elapsed().as_millis() as u64,
            ),
            Err(e) => ComponentCheck::unhealthy(
                "audio",
                format!("Failed to list audio devices: {}", e),
                start.elapsed().as_millis() as u64,
            ),
        }
    }
}

/// Periodic health monitor
pub struct HealthMonitor {
    checker: HealthChecker,
    check_interval: Duration,
    last_check: Option<HealthCheck>,
}

impl HealthMonitor {
    pub fn new(check_interval: Duration) -> Self {
        Self {
            checker: HealthChecker::new(),
            check_interval,
            last_check: None,
        }
    }

    /// Start monitoring in background
    pub async fn start(
        &mut self,
        daemon_state: crate::daemon::DaemonState,
    ) -> tokio::task::JoinHandle<()> {
        let mut interval = tokio::time::interval(self.check_interval);
        let checker = self.checker.clone();

        tokio::spawn(async move {
            loop {
                interval.tick().await;

                let health = checker.check(&daemon_state).await;

                match health.status {
                    HealthStatus::Healthy => {
                        // All good, no logging needed
                    }
                    HealthStatus::Degraded => {
                        warn!("Health check degraded: {:?}", health);
                    }
                    HealthStatus::Unhealthy => {
                        error!("Health check failed: {:?}", health);
                    }
                }
            }
        })
    }

    /// Get last health check result
    pub fn last_check(&self) -> Option<&HealthCheck> {
        self.last_check.as_ref()
    }
}

impl Clone for HealthChecker {
    fn clone(&self) -> Self {
        Self {
            memory_warning_threshold: self.memory_warning_threshold,
            memory_critical_threshold: self.memory_critical_threshold,
            cpu_warning_threshold: self.cpu_warning_threshold,
            cpu_critical_threshold: self.cpu_critical_threshold,
        }
    }
}
