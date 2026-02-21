/**
 * Config data access — reads/writes the onevox config.toml file.
 *
 * Platform paths:
 *   Windows: %APPDATA%\onevox\onevox\config\config.toml
 *   macOS:   ~/Library/Application Support/com.onevox.onevox/config.toml
 *   Linux:   ~/.config/onevox/config.toml
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs"
import { dirname, join } from "node:path"
import { homedir } from "node:os"

// ── TOML helpers (minimal, no external dep) ─────────────────────────────

function parseTOML(text: string): Record<string, any> {
  // We shell out to the Rust CLI for reliable TOML handling:
  //   `onevox config show` prints the live config.
  // For direct parsing we use a best-effort regex approach for flat/nested tables.
  const result: Record<string, any> = {}
  let currentSection: Record<string, any> = result
  let currentKey = ""

  for (const rawLine of text.split("\n")) {
    const line = rawLine.trim()
    if (line === "" || line.startsWith("#")) continue

    // [section]
    const sectionMatch = line.match(/^\[([^\]]+)\]$/)
    if (sectionMatch) {
      const key = sectionMatch[1]
      result[key] = result[key] ?? {}
      currentSection = result[key]
      currentKey = key
      continue
    }

    // key = value
    const kvMatch = line.match(/^(\w+)\s*=\s*(.+)$/)
    if (kvMatch) {
      const [, k, rawVal] = kvMatch
      currentSection[k] = parseValue(rawVal)
    }
  }
  return result
}

function parseValue(raw: string): any {
  const trimmed = raw.trim()
  if (trimmed === "true") return true
  if (trimmed === "false") return false
  if (/^-?\d+$/.test(trimmed)) return parseInt(trimmed, 10)
  if (/^-?\d+\.\d+$/.test(trimmed)) return parseFloat(trimmed)
  if (trimmed.startsWith('"') && trimmed.endsWith('"'))
    return trimmed.slice(1, -1)
  if (trimmed.startsWith("'") && trimmed.endsWith("'"))
    return trimmed.slice(1, -1)
  return trimmed
}

function stringifyTOML(obj: Record<string, any>): string {
  const lines: string[] = []
  // top-level scalars first (unlikely for onevox, but safe)
  for (const [k, v] of Object.entries(obj)) {
    if (typeof v !== "object" || v === null) {
      lines.push(`${k} = ${formatValue(v)}`)
    }
  }
  // sections
  for (const [section, table] of Object.entries(obj)) {
    if (typeof table !== "object" || table === null) continue
    lines.push("")
    lines.push(`[${section}]`)
    for (const [k, v] of Object.entries(table as Record<string, any>)) {
      lines.push(`${k} = ${formatValue(v)}`)
    }
  }
  return lines.join("\n") + "\n"
}

function formatValue(v: any): string {
  if (typeof v === "string") return `"${v}"`
  if (typeof v === "boolean") return v ? "true" : "false"
  if (typeof v === "number") {
    if (Number.isInteger(v)) return v.toString()
    return v.toString()
  }
  return `"${String(v)}"`
}

// ── Types ────────────────────────────────────────────────────────────────

export interface DaemonConfig {
  auto_start: boolean
  log_level: string
}

export interface HotkeyConfig {
  trigger: string
  mode: string
}

export interface AudioConfig {
  device: string
  sample_rate: number
  chunk_duration_ms: number
}

export interface VadConfig {
  enabled: boolean
  backend: string
  threshold: number
  pre_roll_ms: number
  post_roll_ms: number
  min_speech_chunks: number
  min_silence_chunks: number
  adaptive: boolean
}

export interface ModelConfig {
  backend: string
  model_path: string
  device: string
  language: string
  task: string
  preload: boolean
}

export interface PostProcessingConfig {
  auto_punctuation: boolean
  auto_capitalize: boolean
  remove_filler_words: boolean
}

export interface InjectionConfig {
  method: string
  paste_delay_ms: number
  focus_settle_ms: number
}

export interface UiConfig {
  recording_overlay: boolean
  theme: "dark" | "light"
}

export interface VoxConfig {
  daemon: DaemonConfig
  hotkey: HotkeyConfig
  audio: AudioConfig
  vad: VadConfig
  model: ModelConfig
  post_processing: PostProcessingConfig
  injection: InjectionConfig
  ui: UiConfig
}

// ── Defaults ─────────────────────────────────────────────────────────────

export const DEFAULT_CONFIG: VoxConfig = {
  daemon: { auto_start: true, log_level: "info" },
  hotkey: {
    trigger: "Cmd+Shift+0",
    mode: "push-to-talk",
  },
  audio: { device: "default", sample_rate: 16000, chunk_duration_ms: 200 },
  vad: {
    enabled: true,
    backend: "energy",
    threshold: 0.02,
    pre_roll_ms: 300,
    post_roll_ms: 500,
    min_speech_chunks: 2,
    min_silence_chunks: 3,
    adaptive: true,
  },
  model: {
    backend: "whisper_cpp",
    model_path: "ggml-base.en.bin",
    device: "auto",
    language: "en",
    task: "transcribe",
    preload: true,
  },
  post_processing: {
    auto_punctuation: true,
    auto_capitalize: true,
    remove_filler_words: false,
  },
  injection: { method: "accessibility", paste_delay_ms: 50, focus_settle_ms: 80 },
  ui: { recording_overlay: true, theme: "dark" },
}

// ── Path resolution ──────────────────────────────────────────────────────

export function configDir(): string {
  if (process.env.ONEVOX_CONFIG_DIR) return process.env.ONEVOX_CONFIG_DIR
  if (process.env.VOX_CONFIG_DIR) return process.env.VOX_CONFIG_DIR
  if (process.platform === "win32") {
    return join(process.env.APPDATA || join(homedir(), "AppData", "Roaming"), "onevox", "onevox", "config")
  }
  if (process.platform === "darwin") {
    return join(homedir(), "Library", "Application Support", "com.onevox.onevox")
  }
  return join(process.env.XDG_CONFIG_HOME || join(homedir(), ".config"), "onevox")
}

export function configPath(): string {
  return join(configDir(), "config.toml")
}

// ── Load / Save ──────────────────────────────────────────────────────────

export function loadConfig(): VoxConfig {
  const path = configPath()
  if (!existsSync(path)) return { ...DEFAULT_CONFIG }

  try {
    const text = readFileSync(path, "utf-8")
    const raw = parseTOML(text) as any
    return deepMerge(DEFAULT_CONFIG, raw) as VoxConfig
  } catch {
    return { ...DEFAULT_CONFIG }
  }
}

export function saveConfig(config: VoxConfig): void {
  const path = configPath()
  const dir = dirname(path)
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true })
  writeFileSync(path, stringifyTOML(config as any), "utf-8")
}

// ── Utils ────────────────────────────────────────────────────────────────

function deepMerge(defaults: any, overrides: any): any {
  const result = { ...defaults }
  for (const key of Object.keys(overrides)) {
    if (
      typeof defaults[key] === "object" &&
      defaults[key] !== null &&
      typeof overrides[key] === "object" &&
      overrides[key] !== null &&
      !Array.isArray(defaults[key])
    ) {
      result[key] = deepMerge(defaults[key], overrides[key])
    } else {
      result[key] = overrides[key]
    }
  }
  return result
}
