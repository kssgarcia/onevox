# Whisper.cpp Integration Notes

## Current Status: BLOCKED

The whisper.cpp integration via `whisper-rs` is currently blocked due to C++ compiler compatibility issues with macOS 26.2 SDK.

## Build Errors

When attempting to build `whisper-rs` with Metal support (macOS GPU acceleration), the following errors occur:

```
error: too many decimal points in number
error: stray '@' in program
error: invalid suffix "_6" on integer constant
```

These errors occur in:
- Metal framework headers
- CFNetwork framework headers
- vecLib framework headers
- `ggml-metal.m` (Objective-C Metal backend)

## Root Cause

The issue appears to be related to:
1. macOS 26.2 SDK API version macros (e.g., `API_AVAILABLE(macos(10.15))`)
2. C++ compiler (via Nix) not correctly parsing Objective-C syntax
3. Potential version mismatch between SDK and compiler toolchain

## Attempted Solutions

1. ✅ **Installed cmake** via Homebrew (required for whisper.cpp build)
2. ❌ **Tried Metal support**: `whisper-rs = { version = "0.15", features = ["metal"] }`
   - Result: Compilation errors in Metal framework headers
3. ❌ **Tried CPU-only build**: `whisper-rs = "0.15"`
   - Result: Still fails due to SDK header issues

## Current Workaround

For MVP and testing purposes, we've implemented a **Mock Model** that simulates transcription:

```rust
// src/models/mock.rs
pub struct MockModel {
    // Returns fake transcriptions based on audio duration
    // Allows full pipeline testing without actual model
}
```

This allows us to:
- ✅ Test the full audio → VAD → transcription pipeline
- ✅ Validate the ModelRuntime trait abstraction
- ✅ Test CLI commands end-to-end
- ✅ Demonstrate system architecture

## Model Runtime Abstraction

The `ModelRuntime` trait is fully implemented and tested:

```rust
pub trait ModelRuntime: Send + Sync {
    fn load(&mut self, config: ModelConfig) -> Result<()>;
    fn is_loaded(&self) -> bool;
    fn transcribe(&self, samples: &[f32], sample_rate: u32) -> Result<Transcription>;
    fn transcribe_chunk(&self, chunk: &AudioChunk) -> Result<Transcription>;
    fn transcribe_segment(&self, segment: &SpeechSegment) -> Result<Transcription>;
    fn unload(&mut self);
    fn name(&self) -> &str;
    fn info(&self) -> ModelInfo;
}
```

## Alternative Solutions to Consider

### Option 1: Use system C++ compiler
Instead of Nix-provided compiler:
```bash
export CC=/usr/bin/clang
export CXX=/usr/bin/clang++
cargo build --release
```

### Option 2: Use pre-built whisper.cpp
- Download pre-compiled whisper.cpp library
- Use FFI bindings directly (whisper-cpp-sys)
- Skip Metal support initially

### Option 3: Different Rust binding
Try alternative whisper bindings:
- `mutter` - Simpler whisper.cpp bindings
- `rusty-whisper` - Pure Rust implementation (slower)
- `whisper-rs-2` - Alternative whisper.cpp wrapper

### Option 4: Python fallback
Use Python whisper via PyO3:
- `faster-whisper` (optimized Python implementation)
- Call via subprocess or PyO3
- Trade some performance for compatibility

### Option 5: ONNX Runtime
Use ONNX-based whisper models:
- `ort` crate for ONNX runtime
- Pre-converted whisper ONNX models
- CPU-only, but portable

## Recommended Next Steps

1. **Try Option 1 first**: Use system clang instead of Nix compiler
2. **If Option 1 fails**: Proceed with Option 5 (ONNX) for MVP
3. **Once model works**: Optimize with whisper.cpp later
4. **For production**: Build whisper.cpp separately and link dynamically

## Files Created

Despite the build issues, we successfully created:

- ✅ `src/models/runtime.rs` - Model trait abstraction (130 lines)
- ✅ `src/models/mock.rs` - Mock model for testing (120 lines)
- ⏸️ `src/models/whisper.rs.disabled` - Whisper backend (ready when build works)
- ✅ `src/models.rs` - Module exports

## Testing

Full pipeline testing works with mock model:

```bash
# Test audio capture
./target/release/vox test-audio --duration 3

# Test VAD
./target/release/vox test-vad --duration 10

# Test full pipeline (audio → VAD → mock transcription)
./target/release/vox test-transcribe --duration 10
```

## Conclusion

Phase 4 is **functionally complete** with mock model. The architecture is sound and ready for real model integration once build issues are resolved. The mock model proves the design works end-to-end.

**Status**: ✅ Architecture complete, ⏸️ Real model integration blocked by build issues
