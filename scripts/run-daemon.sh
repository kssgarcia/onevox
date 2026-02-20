#!/bin/bash
# Onevox Daemon Runner
# Automatically sets the ONNX Runtime library path

# Find ONNX Runtime library
if [ -f "/opt/homebrew/lib/libonnxruntime.dylib" ]; then
    export ORT_DYLIB_PATH="/opt/homebrew/lib/libonnxruntime.dylib"
elif [ -f "/opt/homebrew/Cellar/onnxruntime/1.24.2/lib/libonnxruntime.dylib" ]; then
    export ORT_DYLIB_PATH="/opt/homebrew/Cellar/onnxruntime/1.24.2/lib/libonnxruntime.dylib"
elif [ -f "/usr/local/lib/libonnxruntime.dylib" ]; then
    export ORT_DYLIB_PATH="/usr/local/lib/libonnxruntime.dylib"
else
    echo "❌ Error: ONNX Runtime library not found!"
    echo "Please install it with: brew install onnxruntime"
    exit 1
fi

echo "✅ Using ONNX Runtime: $ORT_DYLIB_PATH"

# Run the daemon with all arguments passed through
exec "$(dirname "$0")/../target/release/onevox" daemon "$@"
