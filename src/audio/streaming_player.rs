//! Streaming Audio Player
//!
//! Provides continuous audio playback from a queue of audio chunks.
//! Allows TTS synthesis to happen concurrently with playback.

use crate::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Audio chunk with metadata
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// Audio samples (f32, mono)
    pub samples: Vec<f32>,
    /// Sample rate
    pub sample_rate: u32,
}

impl AudioChunk {
    /// Create a new audio chunk
    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        Self {
            samples,
            sample_rate,
        }
    }

    /// Duration in seconds
    pub fn duration_secs(&self) -> f32 {
        self.samples.len() as f32 / self.sample_rate as f32
    }
}

/// Streaming audio player
///
/// Plays audio chunks from a queue continuously, allowing new chunks
/// to be added while playback is ongoing.
pub struct StreamingAudioPlayer {
    /// Channel sender for adding audio chunks
    sender: mpsc::Sender<AudioChunk>,
    /// Join handle for the playback task
    playback_handle: Option<tokio::task::JoinHandle<Result<()>>>,
}

impl StreamingAudioPlayer {
    /// Create a new streaming audio player
    ///
    /// Returns (player, receiver) - the receiver should be passed to start_playback()
    pub fn new() -> (Self, mpsc::Receiver<AudioChunk>) {
        let (sender, receiver) = mpsc::channel(16); // Buffer up to 16 chunks

        (
            Self {
                sender,
                playback_handle: None,
            },
            receiver,
        )
    }

    /// Start playback task
    ///
    /// This spawns a background task that plays audio chunks as they arrive.
    /// The task will continue until the sender is dropped and all chunks are played.
    pub fn start_playback(
        &mut self,
        mut receiver: mpsc::Receiver<AudioChunk>,
        audio_player: std::sync::Arc<tokio::sync::RwLock<crate::audio::AudioPlayer>>,
    ) {
        let handle = tokio::spawn(async move {
            info!("🎵 Streaming audio playback task started");
            let mut total_chunks = 0;
            let mut total_duration = 0.0f32;

            while let Some(chunk) = receiver.recv().await {
                total_chunks += 1;
                total_duration += chunk.duration_secs();

                debug!(
                    "🔊 Playing chunk {} ({:.2}s, {} samples @ {}Hz)",
                    total_chunks,
                    chunk.duration_secs(),
                    chunk.samples.len(),
                    chunk.sample_rate
                );

                // Play the chunk - need to use spawn_blocking because AudioPlayer::play
                // holds non-Send types (cpal::Stream) across await points
                let samples = chunk.samples.clone();
                let sample_rate = chunk.sample_rate;
                let player_clone = Arc::clone(&audio_player);

                let result = tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async {
                        let player = player_clone.read().await;
                        player.play(&samples, sample_rate).await
                    })
                })
                .await;

                match result {
                    Ok(Ok(())) => {
                        debug!("✅ Chunk {} played successfully", total_chunks);
                    }
                    Ok(Err(e)) => {
                        warn!("❌ Failed to play audio chunk: {}", e);
                    }
                    Err(e) => {
                        warn!("❌ Playback task panicked: {}", e);
                    }
                }
            }

            info!(
                "✅ Streaming audio playback complete: {} chunks, {:.2}s total",
                total_chunks, total_duration
            );

            Ok(())
        });

        self.playback_handle = Some(handle);
    }

    /// Queue an audio chunk for playback
    ///
    /// This is non-blocking and returns immediately. The chunk will be played
    /// when it reaches the front of the queue.
    pub async fn queue_chunk(&self, chunk: AudioChunk) -> Result<()> {
        self.sender.send(chunk).await.map_err(|_| {
            crate::Error::Audio("Failed to queue audio chunk - channel closed".to_string())
        })
    }

    /// Wait for all queued audio to finish playing
    ///
    /// Closes the channel (preventing new chunks) and waits for playback to complete.
    pub async fn finish(mut self) -> Result<()> {
        // Drop the sender to signal no more chunks are coming
        drop(self.sender);

        // Wait for playback task to complete
        if let Some(handle) = self.playback_handle.take() {
            handle
                .await
                .map_err(|e| crate::Error::Audio(format!("Playback task failed: {}", e)))?
        } else {
            Ok(())
        }
    }

    /// Get the sender for queuing chunks (useful for sharing across tasks)
    pub fn sender(&self) -> mpsc::Sender<AudioChunk> {
        self.sender.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_chunk_creation() {
        let samples = vec![0.1, 0.2, 0.3];
        let chunk = AudioChunk::new(samples.clone(), 24000);

        assert_eq!(chunk.samples, samples);
        assert_eq!(chunk.sample_rate, 24000);
        assert!((chunk.duration_secs() - 0.000125).abs() < 0.00001);
    }

    #[test]
    fn test_audio_chunk_duration() {
        let samples = vec![0.0; 24000]; // 1 second at 24kHz
        let chunk = AudioChunk::new(samples, 24000);

        assert!((chunk.duration_secs() - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_streaming_player_creation() {
        let (player, _receiver) = StreamingAudioPlayer::new();

        // Should be able to get a sender
        let sender = player.sender();
        assert_eq!(sender.capacity(), 16);
    }

    #[tokio::test]
    async fn test_queue_chunk() {
        let (player, mut receiver) = StreamingAudioPlayer::new();

        let chunk = AudioChunk::new(vec![0.1, 0.2], 24000);
        player.queue_chunk(chunk.clone()).await.unwrap();

        // Should receive the chunk
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.samples, chunk.samples);
        assert_eq!(received.sample_rate, chunk.sample_rate);
    }

    #[tokio::test]
    async fn test_channel_closes_on_drop() {
        let (player, mut receiver) = StreamingAudioPlayer::new();

        let chunk = AudioChunk::new(vec![0.1], 24000);
        player.queue_chunk(chunk).await.unwrap();

        // Drop the player (closes the channel)
        drop(player);

        // Receive the queued chunk
        assert!(receiver.recv().await.is_some());

        // Next recv should return None (channel closed)
        assert!(receiver.recv().await.is_none());
    }
}
