fn main() {
    // On macOS, link against required frameworks
    #[cfg(target_os = "macos")]
    {
        // Link against Accelerate framework (includes BLAS/LAPACK)
        // Note: whisper-rs-sys also links Accelerate, but we need it here for our direct usage
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }

    // On Windows, enable AVX2 optimizations for better CPU performance
    #[cfg(target_os = "windows")]
    {
        // Enable AVX2 instructions for whisper.cpp on Windows
        println!("cargo:rustc-env=CFLAGS=/arch:AVX2 /O2 /fp:fast");
        println!("cargo:rustc-env=CXXFLAGS=/arch:AVX2 /O2 /fp:fast");

        // Set environment hints for whisper.cpp compilation
        println!("cargo:rustc-env=GGML_BLAS=OFF");

        // Rerun if build.rs changes
        println!("cargo:rerun-if-changed=build.rs");
    }
}
