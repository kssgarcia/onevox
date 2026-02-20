# Quick Reference Guide

## Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run binary
cargo run -- <COMMAND>

# Check code (fast compile check)
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## CLI Commands

```bash
# Show version
./target/release/onevox --version

# Show help
./target/release/onevox --help

# Show configuration
./target/release/onevox config show

# Start daemon (placeholder)
./target/release/onevox daemon

# Check status (placeholder)
./target/release/onevox status

# List devices (placeholder)
./target/release/onevox devices list

# List models (placeholder)
./target/release/onevox models list
```

## Project Structure

```
onevox/
├── docs/              # Documentation
│   ├── PLAN.md        # 14-week development plan
│   ├── ARCHITECTURE.md # System design
│   ├── DEPENDENCIES.md # Tech stack
│   ├── PERFORMANCE.md  # Optimization guide
│   └── INITIALIZATION.md # Setup summary
├── src/               # Source code
│   ├── lib.rs         # Library root
│   ├── main.rs        # Binary entry point
│   ├── config.rs      # ✅ Configuration system
│   ├── daemon.rs      # 🚧 Daemon core (next)
│   ├── ipc.rs         # 🚧 IPC server (next)
│   ├── audio.rs       # ⏸️ Audio processing
│   ├── vad.rs         # ⏸️ Voice activity detection
│   ├── models.rs      # ⏸️ Model runtime
│   ├── platform.rs    # ⏸️ Platform integration
│   └── tui.rs         # ⏸️ Terminal UI
├── Cargo.toml         # Dependencies
├── PROGRESS.md        # ✅ Simple progress tracker
└── README.md          # Project overview
```

## Current Status

✅ **Working**:
- Rust 1.93.1 with Edition 2024
- Clean compilation (0 warnings)
- Configuration system with TOML
- CLI interface with clap
- Release binary: 1.3MB

🚧 **Next to Implement**:
1. Daemon core (event loop, state management)
2. IPC server (Unix socket, commands)
3. Logging infrastructure
4. Basic tests

## Configuration

Config file location: `~/.config/onevox/config.toml`

View current config:
```bash
cargo run -- config show
```

Create config from example:
```bash
cp config.example.toml ~/.config/onevox/config.toml
```

## Development Workflow

1. **Make changes** to source files
2. **Check compilation**: `cargo check`
3. **Run tests**: `cargo test`
4. **Test manually**: `cargo run -- <command>`
5. **Update PROGRESS.md** when completing tasks

## Useful Info

- **Rust Version**: 1.93.1
- **Edition**: 2024
- **Primary Dependencies**: tokio, clap, serde, tracing
- **Documentation**: See `docs/` directory
- **Progress**: See `PROGRESS.md`
- **Phase**: 1 of 8 (35% complete)

## Next Steps

See `docs/PLAN.md` for the complete roadmap.

Phase 1 priorities:
1. Implement daemon core with tokio
2. Add IPC server for daemon control
3. Set up structured logging
4. Write initial tests
5. Create GitHub Actions CI

---

**Last Updated**: Feb 20, 2026
