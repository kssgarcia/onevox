# ONEVOX TUI — Architecture & Implementation

> Terminal user interface for the onevox speech-to-text engine, built with
> [OpenTUI](https://github.com/nicholasgasior/opentui) and
> [Bun](https://bun.sh).

---

## Table of Contents

1. [Overview](#overview)
2. [Technology Stack](#technology-stack)
3. [File Architecture](#file-architecture)
4. [Application Structure](#application-structure)
5. [Component Model](#component-model)
6. [Data Layer](#data-layer)
7. [Keyboard Navigation](#keyboard-navigation)
8. [Theme & Visual Design](#theme--visual-design)
9. [Configuration Panel Internals](#configuration-panel-internals)
10. [Rust Integration](#rust-integration)
11. [Current Status](#current-status)
12. [Running the TUI](#running-the-tui)

---

## Overview

The TUI provides a rich terminal interface for configuring and operating the onevox
speech-to-text daemon. It is a **standalone TypeScript application** that runs
under Bun and communicates with the Rust backend either by reading/writing config
files directly or by shelling out to the compiled `onevox` binary for runtime
operations (device listing, model management, daemon status).

The interface follows a **flat, light-themed, minimalist design** with no
borders or box-drawing characters. Everything is expressed through subtle
background tints, bold section headers, and color contrast.

---

## Technology Stack

| Layer         | Technology                             |
| ------------- | -------------------------------------- |
| Runtime       | Bun (TypeScript, ESM)                  |
| UI Framework  | `@opentui/core` — flex-based TUI       |
| Config Format | TOML (hand-rolled parser, no ext dep)  |
| History Store | JSON (flat file)                       |
| Backend CLI   | `onevox` Rust binary via `Bun.spawn()`    |
| Build         | `bun build --target bun`              |

---

## File Architecture

```
tui/
├── package.json                  # Bun project config
├── tsconfig.json                 # TypeScript config
├── src/
│   ├── index.ts                  # Entry point — renderer bootstrap
│   ├── app.ts                    # Root layout, tabs, global keys
│   ├── components/               # Reusable UI widgets
│   │   ├── card.ts               # Flat card (history entries)
│   │   ├── confirm-popup.ts      # Modal confirmation dialog
│   │   ├── key-capture.ts        # Key combination recorder
│   │   ├── stepper.ts            # ◀ value ▶ numeric selector
│   │   └── toggle.ts             # On/Off slide toggle
│   ├── data/                     # Data access & CLI wrapper
│   │   ├── cli.ts                # Shells to `onevox` binary, device listing
│   │   ├── config.ts             # TOML config read/write
│   │   └── history.ts            # JSON history read/write
│   └── panels/                   # Full-screen content panels
│       ├── config.ts             # Settings panel (8 sections)
│       ├── help.ts               # Help overlay (keyboard shortcuts)
│       └── history.ts            # Transcription history list
└── dist/                         # Build output (bun build)
    └── index.js
```

### Layer Responsibilities

| Layer        | Purpose                                                    |
| ------------ | ---------------------------------------------------------- |
| `index.ts`   | Creates the `CliRenderer`, loads persisted state, boots app |
| `app.ts`     | Owns the root layout tree, tab switching, global keys       |
| `panels/`    | Each panel is a self-contained factory function             |
| `components/`| Composable widgets with `focus()`/`blur()` contracts       |
| `data/`      | Pure data I/O — no UI imports, no renderer dependency       |

---

## Application Structure

The app is built entirely with the **factory function pattern**. There are no
classes — every module exports a `create*()` function that takes a renderer and
options, builds a tree of OpenTUI renderables, wires event handlers, and returns
an instance interface.

### Lifecycle

```
index.ts
  └─ createCliRenderer()
  └─ createApp(renderer, config, history)
       ├─ builds: root → headerBox → tabBar → contentArea → statusBar
       ├─ showHistory() → createHistoryPanel()
       ├─ showConfig()  → createConfigPanel()
       └─ global keypress handler (quit, help, save, tab switch)
```

### Root Layout

```
┌─────────────────────────────────────────────────┐
│  ██▀█ ██▄ █ █▀▀ █ █ █▀█ ▀▄▀   v0.1.0          │  ASCIIFontRenderable
│──────────────────────────────────────────────────│
│  History        Config                           │  TabSelectRenderable
│  ▬▬▬▬▬▬▬▬▬▬▬▬                                   │  (underline indicator)
│                                                  │
│  ┄┄┄┄┄┄┄┄┄ content area ┄┄┄┄┄┄┄┄┄┄┄┄           │  Swapped per tab
│                                                  │
│  ^c Quit  ? Help    ● Unsaved changes            │  Status bar
└─────────────────────────────────────────────────┘
```

---

## Component Model

All custom components implement the same contract:

```typescript
interface ComponentInstance {
  root: BoxRenderable     // Mount point — add to parent
  focus(): void           // Show focus indicator (▶ prefix)
  blur(): void            // Remove focus indicator
  // ... component-specific methods
}
```

### Toggle (`components/toggle.ts`)

A horizontal On/Off switch:

```
Adaptive threshold                 Off  ██  On
```

- `Space` toggles (dispatched externally by config panel)
- Mouse click toggles
- Focus shows `▶` before label

### Stepper (`components/stepper.ts`)

A value cycler for predefined option lists:

```
Sample Rate (Hz):                  ◀  16000  ▶
```

- `Left`/`Right` arrows cycle through values (dispatched externally)
- Focus highlights the label and arrows
- Used for: sample rate, chunk duration, VAD threshold, paste delay, max entries

### KeyCapture (`components/key-capture.ts`)

Records key combinations:

```
Push-to-talk trigger:              Ctrl+Shift+Space
```

When focused:

```
▶ Push-to-talk trigger:            ⌨ Press key combo...
```

- Listens via `renderer.keyInput.on("keypress")` — raw key events
- Formats modifier keys: `Ctrl+Shift+Alt+Space`
- `Escape` cancels capture, `Tab` passes through for navigation
- `destroy()` removes the listener

### Card (`components/card.ts`)

Flat history entry card:

```
██ Hello, how are you today?                   ⌘ + ↩ x
   2026-01-15 14:32:00 · whisper-base.en · 1.2s
```

- Selection changes background and indicator color
- Action row supports Copy, Export, Expand, Delete

### ConfirmPopup (`components/confirm-popup.ts`)

Modal confirmation with semi-transparent backdrop:

```
┌─────────────────────────┐
│  Delete entry?          │
│                         │
│  [Yes (y)]  [No (Esc)]  │
└─────────────────────────┘
```

- `y` confirms, `n`/`Esc` cancels
- `prependInputHandler` consumes all input while open

---

## Data Layer

### Config (`data/config.ts`)

- **Path resolution**: `%APPDATA%\onevox\config.toml` (Win), `~/Library/Application Support/onevox/config.toml` (macOS), `~/.config/onevox/config.toml` (Linux)
- **Format**: TOML, parsed with a hand-rolled regex parser (no external dependency)
- **Load**: Deep-merges file values over `DEFAULT_CONFIG` — missing keys get defaults
- **Save**: Serializes each section as `[section]` with `key = value` lines

8 config sections:

| Section            | Key Fields                                     |
| ------------------ | ---------------------------------------------- |
| `daemon`           | `auto_start`, `log_level`                      |
| `hotkey`           | `trigger`, `toggle`, `mode`                    |
| `audio`            | `device`, `sample_rate`, `chunk_duration_ms`   |
| `vad`              | `enabled`, `backend`, `threshold`, `adaptive`  |
| `model`            | `backend`, `model_path`, `device`, `language`  |
| `post_processing`  | `auto_punctuation`, `auto_capitalize`, `remove_filler_words` |
| `injection`        | `method`, `paste_delay_ms`                     |
| `history`          | `enabled`, `max_entries`                       |

### CLI Wrapper (`data/cli.ts`)

Shells out to the compiled `onevox` binary for operations that require the Rust
runtime (audio device enumeration, model download, daemon status).

**Binary resolution** (`voxBin()`):

1. Check `$VOX_BIN` environment variable
2. Walk upward from `import.meta.dir` looking for `Cargo.toml` → use `target/release/onevox` or `target/debug/onevox`
3. Walk upward from `process.cwd()` as fallback
4. Fall back to `"onevox"` on `$PATH`

**Device listing** (`listDevicesWithError()`):

1. Run `onevox devices list`, parse output lines matching `N. DeviceName (default) - 48000Hz, 2 ch`
2. On Windows, if no devices found, fall back to PowerShell `Get-CimInstance Win32_SoundDevice`
3. Return `{ devices, error }` so the UI can display errors

### History (`data/history.ts`)

- **Path**: `%APPDATA%\onevox\history.json` (Win), `~/Library/Application Support/onevox/history.json` (macOS), `~/.local/share/onevox/history.json` (Linux)
- **Format**: JSON array of `HistoryEntry` objects
- Utility functions: `newestFirst()`, `formatTimestamp()`, `formatDuration()`, `truncateText()`

---

## Keyboard Navigation

### Global Keys

| Key      | Action                         |
| -------- | ------------------------------ |
| `Tab`    | Switch History ↔ Config tab    |
| `Ctrl+S` | Save config                   |
| `Ctrl+C` | Quit                          |
| `q`      | Quit (unless modal open)       |
| `?`      | Toggle help overlay            |

### History Panel

| Key         | Action             |
| ----------- | ------------------ |
| `j` / `↓`  | Select next entry  |
| `k` / `↑`  | Select prev entry  |
| `c`         | Copy to clipboard  |
| `e`         | Export to file     |
| `Enter`     | Expand full text   |
| `d`         | Delete entry       |
| `D`         | Clear all          |

### Config Panel — Focus Management

The config panel implements its own focus system because it mixes OpenTUI's
built-in `SelectRenderable` (which has its own focus) with custom widgets that
are not `Renderable`-focusable (toggles, steppers, key-captures).

**Architecture:**

```
renderer.prependInputHandler(configInputHandler)
  ↓ intercepts raw ANSI sequences BEFORE renderer dispatch
  ↓
Tab        → focusNext()    (cycle through focusables[])
Shift+Tab  → focusPrev()
Escape     → blurAll() + return focus to tab bar
Space      → toggle (if current item is a toggle)
Left/Right → prev/next (if current item is a stepper)
```

Each focusable item is tracked in a `focusables: FocusItem[]` array with a
`scrollHint` value. On focus change, `scrollBox.scrollTop` is set to the hint
so the viewport follows the cursor. The scrollbar itself is hidden — scrolling
is entirely driven by keyboard navigation.

**FocusItem types:**

| Type         | Behavior                                              |
| ------------ | ----------------------------------------------------- |
| `select`     | Calls `widget.focus()` — OpenTUI handles Up/Down      |
| `toggle`     | Calls `instance.focus()`, Space toggles                |
| `stepper`    | Calls `instance.focus()`, Left/Right cycle values      |
| `keycapture` | Calls `instance.focus()`, all keys pass through except Tab/Escape |

---

## Theme & Visual Design

Light theme, no borders, minimal chrome.

### Color Palette

| Token                | Hex       | Usage                          |
| -------------------- | --------- | ------------------------------ |
| Background           | `#F5F5F5` | App root, select backgrounds   |
| Primary text         | `#1a1a1a` | Section headers, focused labels |
| Secondary text       | `#555555` | Labels, unfocused text         |
| Muted text           | `#999999` | Descriptions, hints            |
| Selected row         | `#E0E8FF` | Selected option in selects     |
| Focused background   | `#F0F0F0` | Focused select/input area      |
| Success              | `#4CAF50` | Save confirmation, Toggle "On" |
| Error                | `#D32F2F` | Error messages                 |
| Accent blue          | `#2196F3` | Key-capture active state       |
| Status bar           | `#EEEEEE` | Bottom bar background          |
| Dirty indicator      | `#888888` | "Unsaved changes" text         |

### ASCII Title

The `ASCIIFontRenderable` renders "ONEVOX" using the `"block"` font with a
two-color gradient: `[#1a1a1a, #AAAAAA]`.

---

## Configuration Panel Internals

The config panel builds 8 flat sections, each created by `createSection()` which
adds a bold header `TextRenderable` and an indented content `BoxRenderable` to
the scroll container.

### Section → Widget Mapping

| # | Section          | Widgets                                          |
| - | ---------------- | ------------------------------------------------ |
| 1 | Model Selection  | `SelectRenderable` (4 options with descriptions) |
| 2 | Key Bindings     | 2× `KeyCapture` + 1× `SelectRenderable` (mode)  |
| 3 | Device Selection | `SelectRenderable` (async loaded)                |
| 4 | Audio Settings   | 2× `Stepper` (sample rate, chunk duration)       |
| 5 | VAD Settings     | 1× `Toggle` + 1× `Select` + 1× `Stepper` + 1× `Toggle` |
| 6 | Post Processing  | 3× `Toggle`                                     |
| 7 | Text Injection   | 1× `Select` + 1× `Stepper`                      |
| 8 | History Settings | 1× `Toggle` + 1× `Stepper`                      |

### Stepper Value Ranges

| Field            | Values                                          |
| ---------------- | ----------------------------------------------- |
| Sample Rate      | 8000, 11025, 16000, 22050, 44100, 48000, 96000  |
| Chunk Duration   | 50, 100, 150, 200, 300, 400, 500, 1000 ms       |
| VAD Threshold    | 0.00 – 1.00 in steps of 0.01                    |
| Paste Delay      | 0, 10, 20, 30, 50, 75, 100, 150, 200, 300, 500 ms |
| Max Entries      | 100, 200, 500, 1000, 2000, 5000, 10000          |

### Save Flow

```
User presses Ctrl+S
  → app.ts: if configDirty, calls configPanel.save()
  → config.ts: save() calls saveConfig(config) → writes TOML to disk
  → callbacks.onSaved() fires
  → status bar: "✓ Saved" in green (#4CAF50), clears after 3s
  → on failure: "✗ Failed to save: {error}" in status bar
  → if nothing dirty: "Nothing to save" in muted grey, clears after 1.5s
```

---

## Rust Integration

The TUI interacts with the Rust backend in two ways:

### 1. Direct File I/O (config, history)

Config and history files are read/written directly by the TUI's data layer.
The TOML config format is shared with the Rust daemon — both read from the same
`config.toml`. This means changes saved in the TUI are immediately available
when the daemon reloads its config.

### 2. CLI Subprocess (`data/cli.ts`)

For runtime operations that require CPAL or other native libraries, the TUI
shells out to the compiled `onevox` binary:

| Operation       | Command                 | Notes                              |
| --------------- | ----------------------- | ---------------------------------- |
| List devices    | `onevox devices list`      | Parses stdout, Windows PS fallback |
| List models     | `onevox models downloaded` | Checks which models are on disk    |
| Download model  | `onevox models download X` | Triggers download                  |
| Remove model    | `onevox models remove X`   | Deletes model files                |
| Daemon status   | `onevox status`            | Connection check                   |

The binary is located by walking up from the TUI's source directory until
`Cargo.toml` is found, then checking `target/release/onevox` and `target/debug/onevox`.

### IPC Protocol (future)

The Rust codebase defines an IPC protocol (`src/ipc/protocol.rs`) with commands
including `ListDevices`, `ListModels`, `GetConfig`, etc. The server-side handler
for `ListDevices` is currently a stub. When fully implemented, the TUI could
connect to the running daemon via Unix sockets (or named pipes on Windows)
instead of spawning subprocesses.

---

## Current Status

### Working

- [x] Full light theme — flat, borderless, minimalist
- [x] ASCII title with dual-color "block" font
- [x] Tab switching (History / Config) with underline indicator
- [x] History panel — card list with copy, export, expand, delete
- [x] Config panel — 8 sections, all interactive
- [x] Keyboard navigation — Tab/Shift+Tab cycle, Escape returns to tabs
- [x] Select widgets — `SELECTION_CHANGED` updates config immediately
- [x] Toggle widgets — Space bar toggles On/Off
- [x] Stepper widgets — Left/Right cycle through predefined values
- [x] Key capture — records key combinations (Ctrl+Shift+Space format)
- [x] Auto-scroll — viewport follows focused widget, scrollbar hidden
- [x] Save feedback — green "✓ Saved" on Ctrl+S, muted "Nothing to save" otherwise
- [x] Config persistence — TOML read/write, deep-merge with defaults
- [x] History persistence — JSON read/write
- [x] Help overlay — `?` toggles full keyboard shortcut reference
- [x] Device listing — `onevox devices list` with Windows PowerShell fallback
- [x] Binary auto-discovery — walks up to `Cargo.toml` for `target/` binaries
- [x] Error surfacing — device/binary errors shown in UI, not silently swallowed

### Not Yet Implemented

- [ ] IPC-based device listing (server stub exists, not wired)
- [ ] Model download progress UI (CLI wrapper exists, no progress bar)
- [ ] Live daemon status indicator in status bar
- [ ] Real-time transcription display
- [ ] Config reload notification when daemon picks up changes
- [ ] Named pipe IPC on Windows (Unix sockets only in Rust)

---

## Running the TUI

### Prerequisites

- [Bun](https://bun.sh) installed
- Rust project built (`cargo build` or `cargo build --release`)

### Commands

```bash
# From the project root
cd tui

# Install dependencies (first time)
bun install

# Development mode (auto-reload on file changes)
bun dev

# Production run
bun start

# Build bundle
bun build src/index.ts --outdir dist --target bun
```

### Environment Variables

| Variable  | Purpose                                    |
| --------- | ------------------------------------------ |
| `VOX_BIN` | Override path to the `onevox` binary          |

---

*Last updated: February 2026*
