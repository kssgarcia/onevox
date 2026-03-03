fn main() {
    // On macOS, link against required frameworks
    #[cfg(target_os = "macos")]
    {
        // Link against Accelerate framework (includes BLAS/LAPACK)
        // Note: whisper-rs-sys also links Accelerate, but we need it here for our direct usage
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }

    // On Windows, enable AVX2 optimizations and OpenBLAS for 20-30x performance improvement
    #[cfg(target_os = "windows")]
    {
        // Enable AVX2 instructions for whisper.cpp on Windows
        println!("cargo:rustc-env=CFLAGS=/arch:AVX2 /O2 /fp:fast");
        println!("cargo:rustc-env=CXXFLAGS=/arch:AVX2 /O2 /fp:fast");

        // Enable OpenBLAS for optimized matrix operations (critical for performance)
        // Without BLAS, transcription is 20-30x slower (40-60s vs 2s)
        println!("cargo:rustc-env=GGML_BLAS=ON");
        println!("cargo:rustc-env=GGML_OPENBLAS=ON");

        // Rerun if build.rs changes
        println!("cargo:rerun-if-changed=build.rs");
    }
}
