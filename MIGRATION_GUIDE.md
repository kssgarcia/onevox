# Path Migration Guide — v0.1.0+

**Affected Versions:** Users upgrading from pre-v0.1.0 builds  
**Date:** February 2026

---

## What Changed?

Onevox v0.1.0+ now uses **industry-standard platform-appropriate paths** following reverse-DNS naming conventions. This ensures proper namespacing and follows macOS/Linux/Windows best practices.

### macOS Path Changes

| Category | Old Path | New Path |
|----------|----------|----------|
| **Cache** | `~/Library/Caches/onevox` | `~/Library/Caches/com.onevox.onevox` |
| **Config** | `~/Library/Application Support/onevox` | `~/Library/Application Support/com.onevox.onevox` |
| **Logs** | `~/Library/Logs/onevox` | `~/Library/Logs/com.onevox.onevox` |
| **IPC Socket** | `/tmp/onevox.sock` | `/tmp/onevox/onevox.sock` |

### Linux Path Changes

**No changes required!** Linux paths already follow XDG Base Directory specification:
- Cache: `~/.cache/onevox`
- Config: `~/.config/onevox`
- Data: `~/.local/share/onevox`

### What Gets Migrated?

1. **whisper-cli binary** → `~/Library/Caches/com.onevox.onevox/bin/whisper-cli`
2. **AI models** (*.bin files) → `~/Library/Caches/com.onevox.onevox/models/`
3. **config.toml** → `~/Library/Application Support/com.onevox.onevox/config.toml`
4. **history.json** → `~/Library/Application Support/com.onevox.onevox/history.json`

---

## Automatic Migration

### Option 1: Use the Migration Script (Recommended)

```bash
# From the onevox directory
./scripts/migrate_legacy_paths.sh
```

This script will:
- ✅ Detect your platform
- ✅ Copy all data from old → new locations
- ✅ Preserve file permissions and timestamps
- ✅ Verify migration success
- ✅ Provide cleanup instructions

### Option 2: Manual Migration (macOS)

If you prefer to migrate manually:

```bash
# 1. Create new directories
mkdir -p ~/Library/Caches/com.onevox.onevox/{bin,models}
mkdir -p ~/Library/Application\ Support/com.onevox.onevox

# 2. Copy whisper-cli binary
cp ~/Library/Caches/onevox/bin/whisper-cli \
   ~/Library/Caches/com.onevox.onevox/bin/whisper-cli
chmod +x ~/Library/Caches/com.onevox.onevox/bin/whisper-cli

# 3. Copy models
cp -R ~/Library/Caches/onevox/models/* \
   ~/Library/Caches/com.onevox.onevox/models/

# 4. Copy config
cp ~/Library/Application\ Support/onevox/config.toml \
   ~/Library/Application\ Support/com.onevox.onevox/config.toml

# 5. Copy history
cp ~/Library/Application\ Support/onevox/history.json \
   ~/Library/Application\ Support/com.onevox.onevox/history.json
```

---

## Verification

After migration, verify everything works:

```bash
# 1. Check binary is in place
ls -lh ~/Library/Caches/com.onevox.onevox/bin/whisper-cli

# 2. Check models are available
ls ~/Library/Caches/com.onevox.onevox/models/

# 3. Check config exists
cat ~/Library/Application\ Support/com.onevox.onevox/config.toml

# 4. Test daemon startup
onevox daemon --foreground
# Press Ctrl+C to stop

# 5. Check status
onevox status
```

Expected output:
```
✅ whisper-cli binary (807K)
✅ Models: ggml-base.en, whisper-tiny.en
✅ config.toml found
✅ Daemon starts without errors
```

---

## Common Issues

### Issue: "whisper-cli binary not found"

**Symptom:**
```
ERROR Failed to create dictation engine: Model error: whisper-cli binary not found at: 
/Users/YOU/Library/Caches/com.onevox.onevox/bin/whisper-cli
```

**Solution:**
```bash
# Check if binary exists in old location
ls ~/Library/Caches/onevox/bin/whisper-cli

# Copy to new location
mkdir -p ~/Library/Caches/com.onevox.onevox/bin
cp ~/Library/Caches/onevox/bin/whisper-cli \
   ~/Library/Caches/com.onevox.onevox/bin/whisper-cli
chmod +x ~/Library/Caches/com.onevox.onevox/bin/whisper-cli
```

### Issue: "Config file not found"

**Symptom:**
```
WARN Config file not found at ".../com.onevox.onevox/config.toml", using defaults
```

**Solution:**
```bash
# Copy from old location
cp ~/Library/Application\ Support/onevox/config.toml \
   ~/Library/Application\ Support/com.onevox.onevox/config.toml

# Or generate a new one
onevox config init
```

### Issue: "Model not found"

**Symptom:**
```
ERROR Model file not found: ggml-base.en.bin
```

**Solution:**
```bash
# Option 1: Download fresh
onevox models download ggml-base.en

# Option 2: Copy from old location
cp -R ~/Library/Caches/onevox/models/* \
   ~/Library/Caches/com.onevox.onevox/models/
```

---

## Cleanup (Optional)

After verifying everything works, you can remove the old directories:

```bash
# ⚠️  Only run this after verifying migration succeeded!

# macOS
rm -rf ~/Library/Caches/onevox
rm -rf ~/Library/Application\ Support/onevox
rm -rf ~/Library/Logs/onevox

# This will free up disk space by removing duplicate data
```

**Tip:** Keep the old directories for a few days until you're confident everything works.

---

## Fresh Installation

If you're installing Onevox v0.1.0+ for the first time, no migration is needed! The new paths will be created automatically:

```bash
# 1. Install models
onevox models download ggml-base.en

# 2. Start daemon
onevox daemon

# Everything will be created in the correct locations automatically
```

---

## Why This Change?

### Benefits of New Paths

1. **Platform Conventions:** Follows Apple's reverse-DNS naming (`com.onevox.onevox`)
2. **No Conflicts:** Namespaced paths prevent conflicts with other apps named "onevox"
3. **Professional:** Matches how commercial macOS applications organize data
4. **Future-Proof:** Infrastructure ready for Windows/Linux multi-platform support
5. **Security:** Proper permissions (0o700) on sensitive directories

### Industry Standards

- **macOS:** Uses reverse-DNS bundle identifiers (e.g., `com.company.app`)
- **Linux:** Follows XDG Base Directory specification
- **Windows:** Uses `%APPDATA%` and `%LOCALAPPDATA%` properly

---

## Support

If you encounter issues during migration:

1. **Check the migration script output** for error messages
2. **Verify file permissions:** Files should be owned by your user
3. **Check disk space:** Ensure you have enough space for duplicated data
4. **Run daemon in foreground** to see detailed error messages:
   ```bash
   onevox daemon --foreground
   ```

---

## For Developers

If you're building from source, the new paths are managed by the `directories` crate using `ProjectDirs`:

```rust
use crate::platform::paths;

// These automatically resolve to platform-appropriate locations
let cache = paths::cache_dir()?;    // macOS: ~/Library/Caches/com.onevox.onevox
let config = paths::config_dir()?;  // macOS: ~/Library/App Support/com.onevox.onevox
let models = paths::models_dir()?;  // macOS: ~/Library/Caches/com.onevox.onevox/models
```

All path resolution is centralized in `src/platform/paths.rs`.
