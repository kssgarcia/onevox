#!/bin/bash
cd /Users/kevinsepulveda/Documents/onevox
echo "Building with ONNX support..."
cargo build --features onnx --release 2>&1 | tail -5

echo -e "\n\nRunning test with debug output..."
RUST_LOG=onevox::models::onnx_runtime=debug cargo test --features onnx test_onnx_transcription_synthetic_audio --release -- --nocapture 2>&1 | grep -A 15 "Token stats"
