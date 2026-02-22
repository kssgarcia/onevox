//! Model Downloader
//!
//! Downloads Whisper models from Hugging Face with progress tracking.

use crate::models::registry::ModelMetadata;
use anyhow::{Context, Result};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

/// Model downloader
pub struct ModelDownloader {
    cache_dir: PathBuf,
    client: reqwest::Client,
}

impl ModelDownloader {
    /// Create a new model downloader
    pub fn new() -> Result<Self> {
        let cache_dir = crate::platform::models_dir().context("Failed to get models directory")?;

        let client = reqwest::Client::builder()
            .user_agent("onevox/0.1.0")
            .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout
            .connect_timeout(std::time::Duration::from_secs(30)) // 30 second connect timeout
            .pool_max_idle_per_host(10) // Connection pooling
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { cache_dir, client })
    }

    /// Get the model cache directory
    pub fn get_cache_dir() -> Result<PathBuf> {
        crate::platform::models_dir().context("Failed to get models directory")
    }

    /// Get the directory for a specific model
    pub fn model_dir(&self, model_id: &str) -> PathBuf {
        self.cache_dir.join(model_id)
    }

    /// Check if a model is already downloaded
    pub async fn is_downloaded(&self, metadata: &ModelMetadata) -> bool {
        let model_dir = self.model_dir(&metadata.id);

        // Check if all required files exist
        for file in &metadata.files {
            let file_path = model_dir.join(file);
            if !file_path.exists() {
                return false;
            }
        }

        true
    }

    /// Download a model
    pub async fn download(&self, metadata: &ModelMetadata) -> Result<PathBuf> {
        let model_dir = self.model_dir(&metadata.id);

        info!("Downloading model: {} to {:?}", metadata.name, model_dir);

        // Note: GGML models now use native whisper.cpp bindings (no CLI needed)
        // The whisper-rs crate provides direct library access

        // Create model directory
        fs::create_dir_all(&model_dir)
            .await
            .context("Failed to create model directory")?;

        // Download each file
        let urls = metadata.download_urls();
        for (file, url) in urls {
            let file_path = model_dir.join(&file);

            // Create parent directory if it doesn't exist
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .context("Failed to create file parent directory")?;
            }

            // Skip if already exists
            if file_path.exists() {
                info!("File already exists: {}", file);
                continue;
            }

            info!("Downloading: {} from {}", file, url);
            if let Err(e) = self.download_file(&url, &file_path).await {
                return Err(anyhow::anyhow!(
                    "Failed to download '{}' from '{}': {}. \
                     You can manually download this file and place it at '{}'.",
                    file,
                    url,
                    e,
                    file_path.display()
                ));
            }

            // Verify downloaded artifact when checksum is available.
            if let Some(expected_sha) = metadata.file_sha256.get(&file) {
                self.verify_checksum(&file_path, expected_sha).await?;
                info!("Checksum verified: {}", file);
            }
        }

        info!("✅ Model downloaded successfully: {}", metadata.id);
        Ok(model_dir)
    }

    /// Download a single file with progress bar and retry logic
    async fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_BACKOFF: u64 = 1000; // 1 second
        
        let mut last_error = None;
        
        for attempt in 1..=MAX_RETRIES {
            match self.download_file_attempt(url, dest).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < MAX_RETRIES {
                        let backoff_ms = INITIAL_BACKOFF * 2u64.pow(attempt - 1);
                        warn!(
                            "Download attempt {} failed, retrying in {}ms...",
                            attempt, backoff_ms
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Download failed after {} retries", MAX_RETRIES)))
    }
    
    /// Single download attempt
    async fn download_file_attempt(&self, url: &str, dest: &Path) -> Result<()> {
        // Send request
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send download request")?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed with status: {}", response.status());
        }

        // Get file size for progress bar
        let total_size = response.content_length().unwrap_or(0);

        // Create progress bar
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("#>-"),
        );
        pb.set_message(format!(
            "Downloading {}",
            dest.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "model".to_string())
        ));

        // Create temporary file
        let temp_path = dest.with_extension("tmp");
        let mut file = fs::File::create(&temp_path)
            .await
            .context("Failed to create temporary file")?;

        // Download with progress
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read download chunk")?;
            file.write_all(&chunk)
                .await
                .context("Failed to write to file")?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message(format!(
            "Downloaded {}",
            dest.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "model".to_string())
        ));

        // Move temp file to final location
        fs::rename(&temp_path, dest)
            .await
            .context("Failed to move downloaded file")?;

        Ok(())
    }

    async fn verify_checksum(&self, path: &Path, expected_sha256: &str) -> Result<()> {
        let file = path.to_path_buf();
        let expected = expected_sha256.to_ascii_lowercase();

        let actual = tokio::task::spawn_blocking(move || -> Result<String> {
            let file_str = file.to_string_lossy().to_string();

            let candidates: [(&str, Vec<&str>); 3] = [
                ("sha256sum", vec![&file_str]),
                ("shasum", vec!["-a", "256", &file_str]),
                ("openssl", vec!["dgst", "-sha256", &file_str]),
            ];

            for (bin, args) in candidates {
                let output = std::process::Command::new(bin).args(args).output();
                let Ok(output) = output else {
                    continue;
                };

                if !output.status.success() {
                    continue;
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(hash) = parse_sha256_from_output(&stdout) {
                    return Ok(hash.to_ascii_lowercase());
                }
            }

            anyhow::bail!(
                "No checksum tool available (tried: sha256sum, shasum, openssl). \
                 Install one to enable model integrity verification."
            )
        })
        .await??;

        if !actual.eq_ignore_ascii_case(expected_sha256) {
            anyhow::bail!(
                "Checksum mismatch for {}. expected={}, actual={}",
                path.display(),
                expected,
                actual
            );
        }

        Ok(())
    }

    /// Remove a downloaded model
    pub async fn remove(&self, model_id: &str) -> Result<()> {
        let model_dir = self.model_dir(model_id);

        if model_dir.exists() {
            info!("Removing model: {} from {:?}", model_id, model_dir);
            fs::remove_dir_all(&model_dir)
                .await
                .context("Failed to remove model directory")?;
            info!("✅ Model removed: {}", model_id);
        } else {
            warn!("Model not found: {}", model_id);
        }

        Ok(())
    }

    /// List all downloaded models
    pub async fn list_downloaded(&self) -> Result<Vec<String>> {
        if !self.cache_dir.exists() {
            return Ok(vec![]);
        }

        let mut models = vec![];
        let mut entries = fs::read_dir(&self.cache_dir)
            .await
            .context("Failed to read cache directory")?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    models.push(name.to_string());
                }
            }
        }

        Ok(models)
    }

    /// Get the size of a downloaded model
    pub async fn model_size(&self, model_id: &str) -> Result<u64> {
        let model_dir = self.model_dir(model_id);

        if !model_dir.exists() {
            return Ok(0);
        }

        // Recursively calculate total size
        Self::dir_size(&model_dir).await
    }

    /// Recursively calculate directory size
    fn dir_size(
        path: &Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64>> + Send + '_>> {
        Box::pin(async move {
            let mut total_size = 0u64;
            let mut entries = fs::read_dir(path).await?;

            while let Some(entry) = entries.next_entry().await? {
                let metadata = entry.metadata().await?;
                if metadata.is_file() {
                    total_size += metadata.len();
                } else if metadata.is_dir() {
                    total_size += Self::dir_size(&entry.path()).await?;
                }
            }

            Ok(total_size)
        })
    }
}

fn parse_sha256_from_output(output: &str) -> Option<String> {
    if let Some(first) = output.split_whitespace().next() {
        if first.len() == 64 && first.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(first.to_string());
        }
    }

    output
        .split('=')
        .nth(1)
        .map(str::trim)
        .filter(|hash| hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()))
        .map(ToOwned::to_owned)
}

impl Default for ModelDownloader {
    fn default() -> Self {
        Self::new().expect("Failed to create model downloader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_dir() {
        let cache_dir = ModelDownloader::get_cache_dir().unwrap();
        assert!(cache_dir.to_string_lossy().contains("onevox"));
        assert!(cache_dir.to_string_lossy().contains("models"));
    }
}
