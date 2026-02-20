// Whisper.cpp CLI-based model runtime
// Uses the standalone whisper-cli binary for transcription
//
// This backend wraps the whisper.cpp command-line tool, avoiding
// the Rust binding build issues with the macOS beta SDK.

use crate::models::runtime::{ModelConfig, ModelInfo, ModelRuntime, Transcription};
use crate::Result;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;
use tracing::{debug, info, warn};

pub struct WhisperCppCli {
    config: ModelConfig,
    binary_path: PathBuf,
    loaded: bool,
}

impl WhisperCppCli {
    /// Create a new WhisperCppCli backend
    ///
    /// # Arguments
    /// * `binary_path` - Path to whisper-cli binary (default: ~/Library/Caches/vox/bin/whisper-cli)
    pub fn new(binary_path: Option<PathBuf>) -> Self {
        let binary_path = binary_path.unwrap_or_else(|| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("vox/bin/whisper-cli")
        });

        Self {
            config: ModelConfig::default(),
            binary_path,
            loaded: false,
        }
    }

    /// Write audio samples to a temporary WAV file
    fn write_temp_wav(&self, audio: &[f32]) -> Result<NamedTempFile> {
        use hound::{SampleFormat, WavSpec, WavWriter};

        let temp_file = NamedTempFile::new()?;

        let spec = WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(temp_file.path(), spec)
            .map_err(|e| crate::Error::Audio(format!("Failed to create WAV writer: {}", e)))?;

        // Convert f32 samples to i16
        for &sample in audio {
            let sample_i16 = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| crate::Error::Audio(format!("Failed to write audio sample: {}", e)))?;
        }

        writer
            .finalize()
            .map_err(|e| crate::Error::Audio(format!("Failed to finalize WAV file: {}", e)))?;

        Ok(temp_file)
    }

    /// Parse whisper-cli output to extract transcription text
    fn parse_output(&self, output: &str) -> Result<String> {
        // whisper-cli outputs in format:
        // [00:00:00.000 --> 00:00:02.000]   transcribed text here
        // We want to extract just the text parts

        let mut text = String::new();

        for line in output.lines() {
            // Skip empty lines and metadata
            if line.trim().is_empty() {
                continue;
            }

            // Lines with timestamps are in format: [timestamp] text
            // Extract everything after the closing bracket
            if let Some(bracket_end) = line.find(']') {
                let segment_text = line[bracket_end + 1..].trim();
                if !segment_text.is_empty() {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(segment_text);
                }
            }
        }

        Ok(text)
    }
}

impl ModelRuntime for WhisperCppCli {
    fn load(&mut self, config: ModelConfig) -> Result<()> {
        info!("Loading WhisperCppCli model: {}", config.model_path);

        // Check if binary exists
        if !self.binary_path.exists() {
            return Err(crate::Error::Model(format!(
                "whisper-cli binary not found at: {}\nPlease build whisper.cpp and copy the binary to this location",
                self.binary_path.display()
            )));
        }

        // Check if model file exists
        let model_path = PathBuf::from(&config.model_path);
        if !model_path.exists() {
            return Err(crate::Error::Model(format!(
                "Model file not found: {}\nPlease download it with: vox model download",
                model_path.display()
            )));
        }

        self.config = config;
        self.loaded = true;

        debug!("Using whisper-cli binary: {}", self.binary_path.display());
        debug!("Using model file: {}", self.config.model_path);

        info!("WhisperCppCli model loaded successfully");

        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn transcribe(&mut self, samples: &[f32], _sample_rate: u32) -> Result<Transcription> {
        if !self.loaded {
            return Err(crate::Error::Model(
                "Model not loaded. Call load() first.".to_string(),
            ));
        }

        let start = std::time::Instant::now();

        debug!(
            "Transcribing {} audio samples ({:.2}s)",
            samples.len(),
            samples.len() as f32 / 16000.0
        );

        // Write audio to temporary WAV file
        let temp_wav = self.write_temp_wav(samples)?;

        debug!("Wrote audio to: {:?}", temp_wav.path());

        // Build command with all options
        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("-m")
            .arg(&self.config.model_path)
            .arg("-f")
            .arg(temp_wav.path())
            .arg("-np") // No progress output
            .arg("-t")
            .arg(self.config.n_threads.to_string())
            .arg("-l")
            .arg(&self.config.language);

        if self.config.translate {
            cmd.arg("-tr");
        }

        // Run whisper-cli
        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::Error::Model(format!(
                "whisper-cli failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("whisper-cli output:\n{}", stdout);

        // Parse the output
        let text = self.parse_output(&stdout)?;

        if text.is_empty() {
            warn!("Empty transcription result");
        }

        let processing_time_ms = start.elapsed().as_millis() as u64;

        Ok(Transcription {
            text: text.trim().to_string(),
            language: Some(self.config.language.clone()),
            confidence: None,
            processing_time_ms,
            tokens: None,
        })
    }

    fn unload(&mut self) {
        info!("Unloading WhisperCppCli model");
        self.loaded = false;
    }

    fn name(&self) -> &str {
        "whisper-cpp-cli"
    }

    fn info(&self) -> ModelInfo {
        ModelInfo {
            name: "WhisperCppCli".to_string(),
            size_bytes: std::fs::metadata(&self.config.model_path)
                .map(|m| m.len())
                .unwrap_or(0),
            model_type: self
                .config
                .model_path
                .split('/')
                .last()
                .unwrap_or("unknown")
                .to_string(),
            backend: "whisper.cpp (CLI)".to_string(),
            gpu_enabled: false, // CPU-only build
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_output() {
        let cli = WhisperCppCli::new(None);

        let output = r#"
[00:00:00.000 --> 00:00:02.000]   Hello world
[00:00:02.000 --> 00:00:04.000]   This is a test
        "#;

        let result = cli.parse_output(output).unwrap();
        assert_eq!(result, "Hello world This is a test");
    }

    #[test]
    fn test_parse_output_single_line() {
        let cli = WhisperCppCli::new(None);

        let output = "[00:00:00.000 --> 00:00:02.000]   You";

        let result = cli.parse_output(output).unwrap();
        assert_eq!(result, "You");
    }
}
