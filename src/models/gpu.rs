//! GPU Capability Detection
//!
//! Provides runtime detection of GPU acceleration capabilities across platforms.
//! This allows graceful fallback to CPU when GPU features are unavailable.

use tracing::debug;

#[cfg(any(feature = "metal", feature = "cuda", feature = "vulkan"))]
use tracing::{info, warn};

/// GPU backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    /// Apple Metal (macOS M1/M2/M3/M4, some Intel Macs)
    Metal,
    /// NVIDIA CUDA
    Cuda,
    /// Vulkan (cross-platform)
    Vulkan,
    /// OpenBLAS (CPU optimization, not true GPU)
    OpenBlas,
    /// No GPU backend available
    None,
}

/// GPU capability information
#[derive(Debug, Clone)]
pub struct GpuCapabilities {
    /// Whether GPU acceleration is available
    pub available: bool,
    /// The GPU backend that's available
    pub backend: GpuBackend,
    /// Human-readable description
    pub description: String,
}

impl GpuCapabilities {
    /// Detect GPU capabilities at runtime
    pub fn detect() -> Self {
        // Check at compile time which GPU features are enabled
        #[cfg(all(feature = "metal", target_os = "macos"))]
        {
            debug!("Checking Metal GPU availability on macOS");
            if Self::check_metal_available() {
                info!("✅ Metal GPU acceleration detected and available");
                return Self {
                    available: true,
                    backend: GpuBackend::Metal,
                    description: "Apple Metal (GPU)".to_string(),
                };
            } else {
                warn!("⚠️  Metal feature enabled but GPU not available");
            }
        }

        #[cfg(all(feature = "cuda", not(target_os = "macos")))]
        {
            debug!("Checking CUDA GPU availability");
            if Self::check_cuda_available() {
                info!("✅ CUDA GPU acceleration detected and available");
                return Self {
                    available: true,
                    backend: GpuBackend::Cuda,
                    description: "NVIDIA CUDA (GPU)".to_string(),
                };
            } else {
                warn!("⚠️  CUDA feature enabled but GPU not available");
            }
        }

        #[cfg(all(feature = "vulkan", not(any(feature = "metal", feature = "cuda"))))]
        {
            debug!("Checking Vulkan GPU availability");
            if Self::check_vulkan_available() {
                info!("✅ Vulkan GPU acceleration detected and available");
                return Self {
                    available: true,
                    backend: GpuBackend::Vulkan,
                    description: "Vulkan (GPU)".to_string(),
                };
            } else {
                warn!("⚠️  Vulkan feature enabled but GPU not available");
            }
        }

        #[cfg(feature = "openblas")]
        {
            debug!("OpenBLAS CPU optimization available");
            return Self {
                available: false,
                backend: GpuBackend::OpenBlas,
                description: "OpenBLAS (optimized CPU)".to_string(),
            };
        }

        // No GPU backend available
        debug!("No GPU backend compiled into binary");
        Self {
            available: false,
            backend: GpuBackend::None,
            description: "CPU only".to_string(),
        }
    }

    /// Check if Metal is available (macOS)
    #[cfg(all(feature = "metal", target_os = "macos"))]
    fn check_metal_available() -> bool {
        // Metal is available on all Apple Silicon Macs and some Intel Macs with AMD GPUs
        // We can safely assume it's available if we're on macOS and Metal feature is enabled
        // The whisper.cpp library will handle the actual Metal initialization
        true
    }

    /// Check if CUDA is available (Linux/Windows with NVIDIA GPU)
    #[cfg(feature = "cuda")]
    fn check_cuda_available() -> bool {
        // Try to detect CUDA runtime
        // This is a basic check - whisper.cpp will do more thorough validation
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/usr/local/cuda/lib64/libcudart.so").exists()
                || std::path::Path::new("/usr/lib/x86_64-linux-gnu/libcudart.so").exists()
        }
        #[cfg(target_os = "windows")]
        {
            // On Windows, check for CUDA in common locations
            std::path::Path::new("C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA").exists()
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            false
        }
    }

    /// Check if Vulkan is available
    #[cfg(feature = "vulkan")]
    fn check_vulkan_available() -> bool {
        // Try to detect Vulkan runtime
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/usr/lib/x86_64-linux-gnu/libvulkan.so.1").exists()
                || std::path::Path::new("/usr/lib/libvulkan.so.1").exists()
        }
        #[cfg(target_os = "windows")]
        {
            std::path::Path::new("C:\\Windows\\System32\\vulkan-1.dll").exists()
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            false
        }
    }

    /// Get a user-friendly message about GPU status
    pub fn status_message(&self) -> String {
        if self.available {
            format!("🎮 GPU Acceleration: ENABLED ({})", self.description)
        } else {
            match self.backend {
                GpuBackend::OpenBlas => "💻 Using OpenBLAS optimized CPU processing".to_string(),
                GpuBackend::None => "💻 Using CPU processing".to_string(),
                _ => "⚠️  GPU backend compiled but not available - using CPU fallback".to_string(),
            }
        }
    }

    /// Get build instructions for enabling GPU
    pub fn build_instructions(&self) -> Option<String> {
        if self.available {
            return None;
        }

        #[cfg(target_os = "macos")]
        {
            Some(
                "To enable GPU acceleration:\n  cargo build --release --features metal".to_string(),
            )
        }

        #[cfg(all(target_os = "linux", not(feature = "cuda"), not(feature = "vulkan")))]
        {
            Some("To enable GPU acceleration:\n  cargo build --release --features cuda   # For NVIDIA GPUs\n  cargo build --release --features vulkan # For AMD/Intel GPUs".to_string())
        }

        #[cfg(all(target_os = "windows", not(feature = "cuda"), not(feature = "vulkan")))]
        {
            Some("To enable GPU acceleration:\n  cargo build --release --features cuda   # For NVIDIA GPUs\n  cargo build --release --features vulkan # For AMD/Intel GPUs".to_string())
        }

        #[cfg(any(feature = "cuda", feature = "vulkan"))]
        {
            Some("GPU backend compiled but hardware/drivers not detected. Install appropriate drivers.".to_string())
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_detection() {
        let caps = GpuCapabilities::detect();
        println!("GPU Capabilities: {:?}", caps);
        println!("Status: {}", caps.status_message());
        if let Some(instructions) = caps.build_instructions() {
            println!("Instructions:\n{}", instructions);
        }
    }
}
