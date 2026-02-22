fn main() {
    // On macOS, link against required frameworks
    #[cfg(target_os = "macos")]
    {
        // Link against Accelerate framework (includes BLAS/LAPACK)
        println!("cargo:rustc-link-lib=framework=Accelerate");
        
        // Add alias for ILP64 BLAS symbols to regular symbols
        println!("cargo:rustc-link-arg=-Wl,-alias,_cblas_sgemm,_cblas_sgemm$NEWLAPACK$ILP64");
    }
}
