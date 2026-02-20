# Vox - Quick Command Reference

## 🚀 Most Important Commands

```bash
# Run the program (like pnpm dev)
cargo run
[Whisper whisper-tiny.en model loaded! Processed 3.4s of audio. Full inference pipeline coming soon...]

# Run with arguments
cargo run -- --help
cargo run -- config show

# Check if code compiles (FAST! Use this a lot)
cargo check

# Build production binary
cargo build --release

# Run production binary
./target/release/vox
```

---

## 📝 Development Workflow

```bash
# 1. Edit code in src/

# 2. Check it compiles
cargo check

# 3. Run it
cargo run -- config show

# 4. Format code
cargo fmt

# 5. Test it
cargo test
```

---

## 🎯 Common Tasks

| Task | Command |
|------|---------|
| Run program | `cargo run` |
| Run with args | `cargo run -- <args>` |
| Quick check | `cargo check` |
| Format code | `cargo fmt` |
| Lint code | `cargo clippy` |
| Run tests | `cargo test` |
| Clean build | `cargo clean` |
| Production build | `cargo build --release` |

---

## 📦 Cargo ≈ pnpm

| You're used to | Use this instead |
|----------------|------------------|
| `pnpm install` | `cargo build` |
| `pnpm dev` | `cargo run` |
| `pnpm build` | `cargo build --release` |
| `pnpm test` | `cargo test` |
| `pnpm format` | `cargo fmt` |
| `pnpm lint` | `cargo clippy` |

---

## ⚡ Pro Tips

- Use `cargo check` instead of `cargo build` (10x faster!)
- Everything after `--` goes to your program: `cargo run -- --help`
- Production builds are slow (~30s), dev builds are fast (~1s)
- `cargo fmt` before every commit
- `Cargo.toml` = `package.json`
- `target/` = `node_modules/` + `dist/`

---

## 🎪 Your Vox Commands

```bash
# Show all commands
cargo run -- --help

# Show version
cargo run -- --version

# Show config
cargo run -- config show

# Start daemon
cargo run -- daemon

# Check status  
cargo run -- status
```

---

**Remember: `cargo run` = `pnpm dev` 🎯**
