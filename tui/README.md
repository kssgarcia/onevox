# ONEVOX TUI

Terminal User Interface for ONEVOX speech-to-text engine.

Built with [OpenTUI](https://github.com/nicholasgasior/opentui) and [Bun](https://bun.sh).

## Features

- ✨ Beautiful light-themed terminal interface
- 🎯 Full keyboard navigation
- 📝 Configuration management (TOML)
- 📜 Transcription history viewer
- 🎤 Audio device selection
- 🔧 Real-time settings control

## Quick Start

### Prerequisites

- [Bun](https://bun.sh) installed
- ONEVOX Rust project built (`cargo build --release`)

### Installation

```bash
cd tui
bun install
```

### Running

```bash
# Development mode (auto-reload)
bun dev

# Production mode
bun start
```

## Documentation

See [docs/TUI.md](../docs/TUI.md) for complete architecture and implementation details.

## Keyboard Shortcuts

### Global
- `Tab` - Switch between History and Config tabs
- `Ctrl+S` - Save configuration
- `Ctrl+C` / `q` - Quit
- `?` - Toggle help overlay

### History Panel
- `j`/`↓` - Next entry
- `k`/`↑` - Previous entry
- `c` - Copy to clipboard
- `e` - Export to file
- `Enter` - Expand full text
- `d` - Delete entry
- `D` - Clear all

### Config Panel
- `Tab` - Next field
- `Shift+Tab` - Previous field
- `Space` - Toggle switches
- `←`/`→` - Cycle stepper values
- `Esc` - Return to tab bar

## Architecture

```
tui/
├── src/
│   ├── index.ts          # Entry point
│   ├── app.ts            # Root layout & tabs
│   ├── components/       # Reusable UI widgets
│   ├── data/            # Data layer (config, history, CLI)
│   └── panels/          # Full-screen content panels
├── package.json
└── tsconfig.json
```

## Integration with Rust Backend

The TUI communicates with the ONEVOX daemon through:

1. **Direct file I/O** - Reads/writes `config.toml` and `history.json`
2. **CLI subprocess** - Shells out to `vox` binary for device listing, model management

## License

Same as parent project (see [LICENSE](../LICENSE))
