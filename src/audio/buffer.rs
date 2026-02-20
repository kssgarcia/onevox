//! Audio Ring Buffer
//!
//! Lock-free ring buffer for zero-copy audio streaming.

use ringbuf::{traits::*, HeapRb};

/// Audio sample type
pub type Sample = f32;

/// Audio buffer for inter-thread communication
pub struct AudioBuffer {
    producer: ringbuf::HeapProd<Sample>,
    consumer: ringbuf::HeapCons<Sample>,
}

impl AudioBuffer {
    /// Create a new audio buffer with capacity in samples
    pub fn new(capacity: usize) -> Self {
        let rb = HeapRb::<Sample>::new(capacity);
        let (producer, consumer) = rb.split();

        Self { producer, consumer }
    }

    /// Create with default capacity (2 seconds at 16kHz)
    pub fn default_capacity() -> Self {
        Self::new(16000 * 2)
    }

    /// Split into producer and consumer
    pub fn split(self) -> (AudioProducer, AudioConsumer) {
        (
            AudioProducer {
                inner: self.producer,
            },
            AudioConsumer {
                inner: self.consumer,
            },
        )
    }
}

/// Producer end of the audio buffer
pub struct AudioProducer {
    inner: ringbuf::HeapProd<Sample>,
}

impl AudioProducer {
    /// Push samples to the buffer
    /// Returns number of samples actually written
    pub fn push(&mut self, samples: &[Sample]) -> usize {
        self.inner.push_slice(samples)
    }

    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// Get number of free slots
    pub fn free_len(&self) -> usize {
        self.inner.vacant_len()
    }

    /// Get total capacity
    pub fn capacity(&self) -> usize {
        self.inner.capacity().get()
    }
}

/// Consumer end of the audio buffer
pub struct AudioConsumer {
    inner: ringbuf::HeapCons<Sample>,
}

impl AudioConsumer {
    /// Pop samples from the buffer into a destination slice
    /// Returns number of samples actually read
    pub fn pop(&mut self, dest: &mut [Sample]) -> usize {
        self.inner.pop_slice(dest)
    }

    /// Pop samples into a new Vec
    pub fn pop_vec(&mut self, count: usize) -> Vec<Sample> {
        let mut vec = vec![0.0; count];
        let read = self.pop(&mut vec);
        vec.truncate(read);
        vec
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get number of available samples
    pub fn len(&self) -> usize {
        self.inner.occupied_len()
    }

    /// Get total capacity
    pub fn capacity(&self) -> usize {
        self.inner.capacity().get()
    }
}

/// Audio chunk for processing
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// Audio samples (mono, f32)
    pub samples: Vec<Sample>,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Timestamp when chunk was captured
    pub timestamp: std::time::Instant,
}

impl AudioChunk {
    /// Create a new audio chunk
    pub fn new(samples: Vec<Sample>, sample_rate: u32) -> Self {
        Self {
            samples,
            sample_rate,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> f32 {
        self.samples.len() as f32 / self.sample_rate as f32
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        (self.duration_secs() * 1000.0) as u64
    }

    /// Check if chunk is empty
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Get number of samples
    pub fn len(&self) -> usize {
        self.samples.len()
    }
}
