/**
 * History data access — reads/writes onevox history.json.
 *
 * Platform paths:
 *   Windows: %APPDATA%\onevox\history.json
 *   macOS:   ~/Library/Application Support/onevox/history.json
 *   Linux:   ~/.local/share/onevox/history.json
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs"
import { dirname, join } from "node:path"
import { homedir } from "node:os"

// ── Types ────────────────────────────────────────────────────────────────

export interface HistoryEntry {
  id: number
  timestamp: number // Unix epoch seconds
  text: string
  model: string
  duration_ms: number
  confidence: number | null
}

// ── Path resolution ──────────────────────────────────────────────────────

export function dataDir(): string {
  if (process.env.ONEVOX_DATA_DIR) return process.env.ONEVOX_DATA_DIR
  if (process.env.VOX_DATA_DIR) return process.env.VOX_DATA_DIR
  if (process.platform === "win32") {
    return join(process.env.APPDATA || join(homedir(), "AppData", "Roaming"), "onevox")
  }
  if (process.platform === "darwin") {
    return join(homedir(), "Library", "Application Support", "onevox")
  }
  return join(process.env.XDG_DATA_HOME || join(homedir(), ".local", "share"), "onevox")
}

export function historyPath(): string {
  return join(dataDir(), "history.json")
}

// ── Load / Save ──────────────────────────────────────────────────────────

export function loadHistory(): HistoryEntry[] {
  const path = historyPath()
  if (!existsSync(path)) return []

  try {
    const text = readFileSync(path, "utf-8")
    const entries: HistoryEntry[] = JSON.parse(text)
    return Array.isArray(entries) ? entries : []
  } catch {
    return []
  }
}

export function saveHistory(entries: HistoryEntry[]): void {
  const path = historyPath()
  const dir = dirname(path)
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true })
  writeFileSync(path, JSON.stringify(entries, null, 2), "utf-8")
}

export function removeEntry(entries: HistoryEntry[], id: number): HistoryEntry[] {
  return entries.filter((e) => e.id !== id)
}

export function clearHistory(): HistoryEntry[] {
  return []
}

// ── Display helpers ──────────────────────────────────────────────────────

export function formatTimestamp(epoch: number): string {
  const d = new Date(epoch * 1000)
  const pad = (n: number) => n.toString().padStart(2, "0")
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

export function truncateText(text: string, max: number): string {
  if (text.length <= max) return text
  return text.slice(0, max - 1) + "…"
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(1)}s`
}

export function newestFirst(entries: HistoryEntry[]): HistoryEntry[] {
  return [...entries].sort((a, b) => b.timestamp - a.timestamp)
}
