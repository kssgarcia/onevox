fn main() {
    // On macOS, link against required frameworks
    #[cfg(target_os = "macos")]
    {
        // Link against Accelerate framework (includes BLAS/LAPACK)
        // Note: whisper-rs-sys also links Accelerate, but we need it here for our direct usage
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }
}
