# Rust/Cargo Commands Cheat Sheet
## For TypeScript/pnpm Developers

---

## 📦 Package Management (Like pnpm)

| pnpm Command | Cargo Equivalent | What It Does |
|--------------|------------------|--------------|
| `pnpm install` | `cargo build` | Downloads dependencies and compiles |
| `pnpm add <package>` | Edit `Cargo.toml` manually | Add a dependency |
| `pnpm remove <package>` | Remove from `Cargo.toml` | Remove a dependency |
| `pnpm update` | `cargo update` | Update dependencies |
| `pnpm install --frozen-lockfile` | `cargo build --locked` | Use exact lock file |

### Adding Dependencies

**TypeScript/pnpm**:
```bash
pnpm add axios
```

**Rust/Cargo**:
1. Edit `Cargo.toml`:
```toml
[dependencies]
reqwest = "0.11"  # Similar to axios
```
2. Run `cargo build` (auto-installs)

---

## 🏗️ Building & Running

| npm/pnpm Command | Cargo Equivalent | What It Does |
|------------------|------------------|--------------|
| `pnpm dev` | `cargo run` | Run in development mode |
| `pnpm build` | `cargo build --release` | Build optimized production binary |
| `pnpm start` | `./target/release/onevox` | Run production binary |
| `tsc --noEmit` | `cargo check` | Type-check without building |

### Common Build Commands

```bash
# Development build (fast, unoptimized)
cargo build

# Run immediately (builds if needed)
cargo run

# Run with arguments
cargo run -- --help
cargo run -- config show
cargo run -- daemon

# Production build (slow, optimized)
cargo build --release

# Run production binary
./target/release/onevox --help
```

---

## 🧪 Testing

| Jest/Vitest Command | Cargo Equivalent | What It Does |
|---------------------|------------------|--------------|
| `pnpm test` | `cargo test` | Run all tests |
| `pnpm test --watch` | `cargo watch -x test` | Run tests on file change |
| `pnpm test:coverage` | `cargo tarpaulin` | Code coverage (needs install) |

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=4
```

---

## 🔍 Code Quality

| ESLint/Prettier Command | Cargo Equivalent | What It Does |
|-------------------------|------------------|--------------|
| `pnpm lint` | `cargo clippy` | Lint code (like ESLint) |
| `pnpm format` | `cargo fmt` | Format code (like Prettier) |
| `pnpm type-check` | `cargo check` | Type checking |

```bash
# Format code (like prettier)
cargo fmt

# Check formatting without changing
cargo fmt -- --check

# Lint code (like eslint)
cargo clippy

# Strict linting
cargo clippy -- -D warnings

# Fast compilation check (no binary)
cargo check
```

---

## 🧹 Cleaning

| npm/pnpm Command | Cargo Equivalent | What It Does |
|------------------|------------------|--------------|
| `rm -rf node_modules` | `cargo clean` | Remove build artifacts |
| N/A | `cargo clean --release` | Clean only release builds |

```bash
# Clean all build artifacts (~1.6GB freed)
cargo clean

# Clean and rebuild
cargo clean && cargo build
```

---

## 📊 Project Info

| npm Command | Cargo Equivalent | What It Does |
|-------------|------------------|--------------|
| `npm list` | `cargo tree` | Show dependency tree |
| `npm outdated` | `cargo outdated` | Check for updates (needs install) |
| `npm audit` | `cargo audit` | Security audit (needs install) |

```bash
# Show dependency tree
cargo tree

# Show what would be updated
cargo update --dry-run

# Update dependencies
cargo update
```

---

## 🚀 Running Your Onevox Project

### Development Workflow

```bash
# 1. Make code changes in src/

# 2. Quick check (fast!)
cargo check

# 3. Run the program
cargo run -- --help

# 4. Run with specific command
cargo run -- config show
cargo run -- daemon

# 5. Run tests
cargo test

# 6. Format code
cargo fmt

# 7. Lint code
cargo clippy
```

### Production Build

```bash
# Build optimized binary (takes ~30s)
cargo build --release

# Binary is at: ./target/release/onevox
# Size: ~1.3MB

# Run it
./target/release/onevox --version
./target/release/onevox config show
```

---

## 🎯 Most Used Commands (Daily)

```bash
# Quick iteration during development
cargo check          # Fast type-check (use this A LOT)
cargo run            # Run your app
cargo test           # Run tests
cargo fmt            # Format code
cargo clippy         # Lint

# When ready to test production
cargo build --release
./target/release/onevox
```

---

## 🔧 Project-Specific Commands (Onevox)

```bash
# Show help
cargo run -- --help

# Show configuration
cargo run -- config show

# Start daemon (placeholder for now)
cargo run -- daemon

# Check daemon status
cargo run -- status

# List audio devices (placeholder)
cargo run -- devices list

# Manage models (placeholder)
cargo run -- models list
```

---

## 📁 File Structure (vs TypeScript)

```
TypeScript Project          Rust Project
├── package.json      →     ├── Cargo.toml
├── pnpm-lock.yaml    →     ├── Cargo.lock
├── node_modules/     →     ├── target/
├── src/              →     ├── src/
│   ├── index.ts      →     │   ├── main.rs (binary)
│   └── lib.ts        →     │   └── lib.rs (library)
├── tsconfig.json     →     (built into Cargo.toml)
└── dist/             →     └── target/release/
```

---

## ⚡ Key Differences

### TypeScript/pnpm
```bash
# Install deps first
pnpm install

# Then run
pnpm dev
```

### Rust/Cargo
```bash
# Just run! (auto-installs deps)
cargo run
```

### TypeScript
```typescript
// Imports
import { Config } from './config';
```

### Rust
```rust
// Imports
use onevox::Config;
```

---

## 🛠️ Useful Tools (Optional)

```bash
# Install cargo-watch (like nodemon)
cargo install cargo-watch

# Auto-run on file changes
cargo watch -x run
cargo watch -x test

# Install cargo-edit (easier dependency management)
cargo install cargo-edit

# Now you can:
cargo add tokio
cargo rm tokio
cargo upgrade
```

---

## 💡 Pro Tips

1. **Use `cargo check` frequently** - It's MUCH faster than `cargo build`
2. **`cargo run --` passes args** - Everything after `--` goes to your program
3. **Release builds are SLOW** - Only use `--release` when you need speed
4. **`cargo fmt` before commit** - Like running Prettier
5. **`cargo clippy` catches bugs** - Better than basic compiler warnings

---

## 🆘 Common Issues

### Problem: "error: linker `cc` failed"
**Solution**: Install build tools
```bash
# macOS
xcode-select --install

# Linux
sudo apt install build-essential
```

### Problem: "Blocking waiting for file lock"
**Solution**: Another cargo process is running
```bash
# Kill it or wait, or:
rm -rf ~/.cargo/.package-cache
```

### Problem: Slow compilation
**Solution**: Use `cargo check` instead of `cargo build`

---

## 📚 Quick Reference Card

```bash
# THE BIG 5 (use these daily)
cargo check          # Fast compile check
cargo run            # Run program
cargo test           # Run tests
cargo fmt            # Format code
cargo clippy         # Lint code

# Building
cargo build          # Debug build
cargo build --release # Production build

# Running
cargo run -- <args>  # Run with arguments
./target/release/onevox # Run production binary

# Cleaning
cargo clean          # Delete build artifacts

# Dependencies
cargo update         # Update deps
cargo tree           # Show dep tree
```

---

## 🎓 Your First Hour Workflow

```bash
# 1. Open project
cd ~/Documents/onevox

# 2. Make sure it compiles
cargo check

# 3. Run it
cargo run -- --help

# 4. Make a change to src/main.rs

# 5. Check if it compiles (FAST!)
cargo check

# 6. Run it
cargo run -- config show

# 7. Format your code
cargo fmt

# 8. Lint it
cargo clippy

# 9. Run tests
cargo test

# 10. Build for production
cargo build --release

# 11. Test production binary
./target/release/onevox --version
```

---

**Save this file and reference it anytime!**

The main thing to remember: **`cargo run`** is like **`pnpm dev`** 🚀
