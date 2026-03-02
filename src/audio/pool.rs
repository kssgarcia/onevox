//! Audio Buffer Pool
//!
//! Reusable buffer pool to reduce allocations during audio capture.

use parking_lot::Mutex;
use std::sync::Arc;

/// Pool of reusable audio buffers
#[derive(Clone)]
pub struct BufferPool {
    buffers: Arc<Mutex<Vec<Vec<f32>>>>,
    buffer_size: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    ///
    /// # Arguments
    /// * `buffer_size` - Size of each buffer in samples
    /// * `initial_capacity` - Number of buffers to pre-allocate
    pub fn new(buffer_size: usize, initial_capacity: usize) -> Self {
        let buffers: Vec<Vec<f32>> = (0..initial_capacity)
            .map(|_| Vec::with_capacity(buffer_size))
            .collect();

        Self {
            buffers: Arc::new(Mutex::new(buffers)),
            buffer_size,
        }
    }

    /// Get a buffer from the pool, or create a new one if pool is empty
    pub fn acquire(&self) -> Vec<f32> {
        self.buffers
            .lock()
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(self.buffer_size))
    }

    /// Return a buffer to the pool for reuse
    pub fn release(&self, mut buffer: Vec<f32>) {
        buffer.clear();
        // Only keep buffers with reasonable capacity to avoid memory bloat
        if buffer.capacity() >= self.buffer_size && buffer.capacity() < self.buffer_size * 2 {
            let mut pool = self.buffers.lock();
            // Limit pool size to prevent unbounded growth
            if pool.len() < 32 {
                pool.push(buffer);
            }
        }
    }

    /// Get current pool size (for debugging/monitoring)
    pub fn size(&self) -> usize {
        self.buffers.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_acquire_release() {
        let pool = BufferPool::new(1024, 4);

        // Pool should start with 4 buffers
        assert_eq!(pool.size(), 4);

        // Acquire a buffer
        let buf = pool.acquire();
        assert_eq!(buf.capacity(), 1024);
        assert_eq!(pool.size(), 3);

        // Release it back
        pool.release(buf);
        assert_eq!(pool.size(), 4);
    }

    #[test]
    fn test_buffer_pool_clears_on_release() {
        let pool = BufferPool::new(1024, 2);

        let mut buf = pool.acquire();
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);

        pool.release(buf);

        let buf2 = pool.acquire();
        assert_eq!(buf2.len(), 0);
        assert!(buf2.capacity() >= 1024);
    }

    #[test]
    fn test_buffer_pool_creates_new_when_empty() {
        let pool = BufferPool::new(1024, 0);
        assert_eq!(pool.size(), 0);

        let buf = pool.acquire();
        assert_eq!(buf.capacity(), 1024);
    }
}
