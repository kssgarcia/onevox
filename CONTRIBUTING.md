# Contributing to OneVox

Thank you for your interest in contributing to OneVox! This document provides guidelines and information for contributors.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/onevox.git`
3. Create a branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Test thoroughly
6. Submit a pull request

## Development Setup

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed build instructions.

**Quick start:**
```bash
# Clone and build
git clone https://github.com/kssgarcia/onevox.git
cd onevox
cargo build --release

# Run tests
cargo test

# Run in foreground for testing
./target/release/onevox daemon --foreground
```

## Code Style

- Follow Rust standard formatting: `cargo fmt`
- Run clippy before committing: `cargo clippy`
- Write clear commit messages
- Add tests for new features
- Update documentation as needed

## Pull Request Process

1. **Update documentation** - If you change functionality, update relevant .md files
2. **Add tests** - New features should include tests
3. **Run checks** - Ensure `cargo test`, `cargo fmt`, and `cargo clippy` pass
4. **Test build variants** - If touching model code, test both whisper.cpp and ONNX builds
5. **Describe changes** - Write a clear PR description explaining what and why
6. **Link issues** - Reference any related issues

**For model backend changes:**
- Test both `cargo build --release` and `cargo build --release --features onnx`
- Verify existing tests pass with both backends
- Update ARCHITECTURE.md if behavior changes

## Areas for Contribution

### High Priority
- Windows installer and service management
- Additional Whisper model support
- ONNX backend improvements and testing
- Performance optimizations
- Cross-platform testing (especially with ONNX backend)
- Documentation improvements

### Feature Ideas
- Additional ONNX-optimized multilingual models
- Custom vocabulary/word lists
- Multiple language support improvements
- Alternative VAD implementations
- Plugin system for post-processing
- Cloud sync for history (optional, privacy-preserving)

### Backend Development
- Testing and stabilizing ONNX backend
- Adding support for new model formats
- Implementing Candle backend (pure Rust)
- GPU optimization for ONNX models
- Model quantization improvements

### Bug Fixes
- Check [Issues](https://github.com/kssgarcia/onevox/issues) for open bugs
- Platform-specific issues are especially valuable

## Architecture Guidelines

OneVox follows these principles:

1. **Privacy First** - All processing must remain local
2. **Cross-Platform** - Changes should work on macOS, Linux, and Windows
3. **Performance** - Minimize latency and resource usage
4. **Simplicity** - Prefer simple, maintainable solutions
5. **Stability** - Production reliability over experimental features

See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details.

## Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# Benchmarks
cargo bench

# Test with different backends
cargo test --features onnx

# Platform-specific tests
./target/release/onevox test-hotkey
./target/release/onevox test-audio --duration 3
./target/release/onevox test-vad --duration 10
```

**Testing Build Variants:**

When contributing changes that affect model backends:

```bash
# Test default backend (whisper.cpp)
cargo build --release
cargo test
./target/release/onevox daemon --foreground

# Test ONNX backend (if applicable)
cargo build --release --features onnx
cargo test --features onnx
# Edit config: model_path = "parakeet-ctc-0.6b"
./target/release/onevox daemon --foreground

# Test with GPU features (if available)
cargo build --release --features metal  # macOS
cargo build --release --features cuda   # NVIDIA
cargo test --features metal
```

## Documentation

When adding features:
- Update README.md if it affects user-facing functionality
- Update INSTALLATION.md for setup changes
- Update QUICKREF.md for new commands
- Add inline code comments for complex logic
- Update ARCHITECTURE.md for design changes

## Commit Messages

Use clear, descriptive commit messages:

```
Good:
- "Add Windows service management support"
- "Fix audio device enumeration on Linux"
- "Optimize VAD processing for lower latency"

Bad:
- "Fix bug"
- "Update code"
- "Changes"
```

## Code Review

All submissions require review. We'll provide feedback on:
- Code quality and style
- Test coverage
- Documentation
- Cross-platform compatibility
- Performance implications

## Community

- Be respectful and constructive
- Help others in issues and discussions
- Share your use cases and feedback
- Report bugs with detailed information

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

- Open an issue for bugs or feature requests
- Start a discussion for questions or ideas
- Check existing issues before creating new ones

Thank you for contributing to OneVox!
