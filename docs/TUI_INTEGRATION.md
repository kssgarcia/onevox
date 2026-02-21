# TUI Integration Guide

This guide explains how to use the ONEVOX Terminal User Interface (TUI) with your Rust application.

## Overview

ONEVOX includes a beautiful terminal user interface built with **OpenTUI** and **TypeScript/Bun**. The TUI provides:

- 🎨 **Light-themed, minimalist design** - Modern, borderless interface
- ⚙️ **Interactive configuration** - 8 sections covering all daemon settings
- 📜 **History viewer** - Browse, copy, export transcriptions
- 🎤 **Device selection** - Visual audio device picker
- ⌨️ **Full keyboard navigation** - Fast, efficient workflow

## Quick Start

### Method 1: Using the Rust CLI (Recommended)

The simplest way to launch the TUI is through the Rust binary:

```bash
# Build the Rust binary first
cargo build --release

# Launch the TUI
./target/release/onevox tui
```

Or if you have `onevox` in your PATH:

```bash
onevox tui
```

**What happens:**
1. The Rust binary checks if Bun is installed
2. Finds the `tui/` directory relative to the binary
3. Installs dependencies if needed (`bun install`)
4. Launches the TypeScript TUI

### Method 2: Direct TUI Launch

You can also run the TUI directly using Bun:

```bash
cd tui
bun install
bun start
```

Or use the helper script:

```bash
./scripts/run-tui.sh
```

## Prerequisites

### Install Bun

The TUI requires [Bun](https://bun.sh) to run. Install it with:

**macOS/Linux:**
```bash
curl -fsSL https://bun.sh/install | bash
```

**Windows:**
```powershell
powershell -c "irm bun.sh/install.ps1 | iex"
```

**Verify installation:**
```bash
bun --version
```

### Build the Rust Binary

```bash
cargo build --release
```

The TUI interacts with the Rust binary for:
- Listing audio devices
- Managing models (download, remove)
- Checking daemon status

## How It Works

### Architecture

```
┌─────────────────────────────────────────┐
│         onevox tui (Rust CLI)          │
│                                         │
│  1. Finds tui/ directory                │
│  2. Checks Bun installation             │
│  3. Runs: bun run src/index.ts          │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│      OpenTUI TypeScript App             │
│                                         │
│  ├─ panels/config.ts   (settings)       │
│  ├─ panels/history.ts  (transcriptions) │
│  ├─ panels/help.ts     (shortcuts)      │
│  ├─ data/config.ts     (TOML I/O)       │
│  ├─ data/history.ts    (JSON I/O)       │
│  └─ data/cli.ts        (onevox binary)     │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│      Rust Backend Integration           │
│                                         │
│  ├─ config.toml  (shared file)          │
│  ├─ history.json (shared file)          │
│  └─ onevox binary   (CLI commands)         │
└─────────────────────────────────────────┘
```

### Communication Methods

The TUI communicates with the Rust backend through:

1. **Shared Configuration File** (`config.toml`)
   - TUI reads/writes TOML directly
   - Daemon reloads config when changed
   - Location: `~/.config/onevox/config.toml` (macOS/Linux)

2. **Shared History File** (`history.json`)
   - TUI displays transcription history
   - Daemon appends new transcriptions
   - Location: `~/.local/share/onevox/history.json`

3. **CLI Subprocess Calls**
   - `onevox devices list` - Get audio devices
   - `onevox models downloaded` - Check installed models
   - `onevox models download <id>` - Download model
   - `onevox status` - Daemon status check

## Using the TUI

### Keyboard Shortcuts

#### Global
- `Tab` - Switch between History and Config tabs
- `Ctrl+S` - Save configuration
- `Ctrl+C` / `q` - Quit
- `?` - Toggle help overlay

#### History Panel
- `j` / `↓` - Next entry
- `k` / `↑` - Previous entry
- `c` - Copy to clipboard
- `e` - Export to file
- `Enter` - Expand full text
- `d` - Delete entry
- `D` - Clear all history

#### Config Panel
- `Tab` - Next field
- `Shift+Tab` - Previous field
- `Space` - Toggle switches (On/Off)
- `←` / `→` - Cycle stepper values
- `↑` / `↓` - Navigate select menus
- `Enter` - Confirm selection
- `Esc` - Return to tab bar

### Configuration Sections

The Config panel has 8 sections:

1. **Model Selection** - Choose Whisper model variant
2. **Key Bindings** - Set hotkeys for push-to-talk
3. **Device Selection** - Pick audio input device
4. **Audio Settings** - Sample rate, chunk duration
5. **VAD Settings** - Voice activity detection params
6. **Post Processing** - Punctuation, capitalization
7. **Text Injection** - Method and paste delay
8. **History Settings** - Enable/disable, max entries

### Workflow Example

```bash
# 1. Launch TUI
onevox tui

# 2. Navigate to Config tab (Tab key)

# 3. Select audio device
#    - Tab to "Device Selection" section
#    - Use ↑/↓ to choose device
#    - Press Enter

# 4. Adjust VAD threshold
#    - Tab to "VAD Settings"
#    - Use ←/→ to adjust threshold

# 5. Save changes
#    - Press Ctrl+S

# 6. Start daemon with new settings
#    - Quit TUI (Ctrl+C)
#    - Run: onevox daemon --foreground

# 7. View transcription history
#    - onevox tui
#    - History tab shows all transcriptions
```

## Development

### TUI Development Mode

For TUI development with auto-reload:

```bash
cd tui
bun --watch src/index.ts
```

This watches for file changes and restarts the TUI automatically.

### Modifying the TUI

The TUI source is in `tui/src/`:

```
tui/src/
├── index.ts              # Entry point
├── app.ts                # Root layout, tabs
├── components/           # Reusable widgets
│   ├── toggle.ts         # On/Off switch
│   ├── stepper.ts        # Value cycler
│   ├── key-capture.ts    # Hotkey recorder
│   ├── card.ts           # History entry card
│   └── confirm-popup.ts  # Modal dialog
├── data/                 # Data layer
│   ├── config.ts         # TOML config I/O
│   ├── history.ts        # JSON history I/O
│   └── cli.ts            # Rust binary wrapper
└── panels/               # Full-screen panels
    ├── config.ts         # Settings panel
    ├── history.ts        # History panel
    └── help.ts           # Keyboard shortcuts
```

### Adding New Config Options

To add a new configuration option:

1. **Update Rust config struct** (`src/config.rs`)
2. **Update TOML serialization** (handled automatically by serde)
3. **Add TUI widget** in `tui/src/panels/config.ts`
4. **Update DEFAULT_CONFIG** in `tui/src/data/config.ts`

See `docs/TUI.md` for detailed TUI architecture docs.

## Troubleshooting

### "Bun is not installed"

```bash
# Install Bun
curl -fsSL https://bun.sh/install | bash

# Reload shell
exec $SHELL

# Verify
bun --version
```

### "Could not find TUI directory"

Make sure you're running from the project root, or the binary can find the `tui/` directory:

```bash
# Check current directory
pwd

# Should show: /path/to/onevox

# Check TUI exists
ls -la tui/
```

If running an installed binary (`/usr/local/bin/onevox`), the TUI finder walks up from the binary location. Make sure the symlink preserves the path:

```bash
# Create symlink that preserves path
sudo ln -sf /Users/you/Documents/onevox/target/release/onevox /usr/local/bin/onevox
```

### "Failed to install dependencies"

```bash
# Manually install
cd tui
rm -rf node_modules
bun install

# Check for errors
bun install --verbose
```

### TUI crashes on startup

```bash
# Run directly to see errors
cd tui
bun run src/index.ts

# Check logs
# Look for TypeScript errors or missing dependencies
```

### Device list is empty

The TUI shells out to `onevox devices list`. Make sure:

1. The Rust binary is built and in PATH
2. Audio permissions are granted (macOS: System Settings → Privacy → Microphone)

```bash
# Test device listing directly
onevox devices list

# If empty, check permissions
# macOS: System Settings → Privacy & Security → Microphone
```

## Environment Variables

### `VOX_BIN`

Override the path to the `onevox` binary:

```bash
export VOX_BIN=/path/to/onevox
onevox tui
```

This is useful if the binary is in a non-standard location.

## Next Steps

- **Read full TUI docs:** `docs/TUI.md`
- **Explore keyboard shortcuts:** Press `?` in the TUI
- **Customize config:** Edit `~/.config/onevox/config.toml`
- **View history:** Check `~/.local/share/onevox/history.json`

## Contributing

See `CONTRIBUTING.md` for TUI development guidelines.

---

**Questions?** Open an issue on GitHub or check the docs!
