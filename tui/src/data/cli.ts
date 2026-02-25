/**
 * CLI wrapper — calls the `onevox` binary for runtime operations
 * that can't be done via file I/O (device listing, model management, etc.)
 */

import { join, dirname } from "node:path"
import { existsSync } from "node:fs"

// ── Types ────────────────────────────────────────────────────────────────

export interface AudioDevice {
  index: number
  name: string
  isDefault: boolean
  sampleRate: number
  channels: number
}

export interface ModelInfo {
  id: string
  name: string
  size: string
  sizeBytes: number
  speedFactor: number
  memoryMb: number
  description: string
  downloaded: boolean
}

export interface DaemonStatus {
  version: string
  state: string
  modelLoaded: boolean
  modelName: string | null
  isDictating: boolean
}

export interface ReloadResult {
  state: "reloaded" | "not_running" | "failed"
  message: string
}

// ── Resolve binary path ──────────────────────────────────────────────────

/** Walk upward from `start` until we find a directory containing Cargo.toml */
function findProjectRoot(start: string): string | null {
  let dir = start
  for (let i = 0; i < 8; i++) {
    if (existsSync(join(dir, "Cargo.toml"))) return dir
    const parent = dirname(dir)
    if (parent === dir) break // filesystem root
    dir = parent
  }
  return null
}

function onevoxBin(): string | null {
  const override = process.env.ONEVOX_BIN || process.env.VOX_BIN
  if (override) {
    return existsSync(override) ? override : null
  }
  const ext = process.platform === "win32" ? ".exe" : ""
  // Try to locate project root by finding Cargo.toml, searching from the
  // module's directory and also from the process working directory.
  const candidateRoots = [
    findProjectRoot(import.meta.dir),
    findProjectRoot(process.cwd()),
  ].filter(Boolean) as string[]

  for (const root of candidateRoots) {
    const release = join(root, "target", "release", `onevox${ext}`)
    const debug   = join(root, "target", "debug",   `onevox${ext}`)
    if (existsSync(release)) return release
    if (existsSync(debug))   return debug
  }
  // Last-resort: hope it's on PATH
  return "onevox"
}

// ── Run a CLI command ────────────────────────────────────────────────────

async function run(args: string[]): Promise<string> {
  const bin = onevoxBin()
  if (!bin) throw new Error("onevox binary not found")
  const proc = Bun.spawn([bin, ...args], {
    stdout: "pipe",
    stderr: "pipe",
  })
  const [text, errText] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ])
  await proc.exited
  if (proc.exitCode !== 0) {
    throw new Error(errText.trim() || `onevox exited with code ${proc.exitCode}`)
  }
  return text.trim()
}

// ── Device listing ───────────────────────────────────────────────────────

function parseDeviceLines(out: string): AudioDevice[] {
  const devices: AudioDevice[] = []
  const lines = out.split("\n").filter((l) => l.trim().length > 0)
  for (const line of lines) {
    // Format: "  1. DeviceName (default) - 48000Hz, 2 ch"
    const match = line.match(/^\s*(\d+)\.\s+(.+)$/)
    if (match) {
      const [, idx, rest] = match
      const isDefault = rest.includes("(default)")
      const name = rest
        .replace("(default)", "")
        .replace(/\s*-\s*\d+Hz.*$/, "")
        .trim()
      const rateMatch = rest.match(/(\d+)Hz/)
      const chMatch = rest.match(/(\d+)\s*ch/)
      devices.push({
        index: parseInt(idx),
        name,
        isDefault,
        sampleRate: rateMatch ? parseInt(rateMatch[1]) : 48000,
        channels: chMatch ? parseInt(chMatch[1]) : 1,
      })
    }
  }
  return devices
}

/** Returns devices plus any error string so callers can show it to the user. */
export async function listDevicesWithError(): Promise<{ devices: AudioDevice[]; error: string | null }> {
  try {
    const out = await run(["devices", "list"])
    const devices = parseDeviceLines(out)
    if (devices.length === 0 && process.platform === "win32") {
      // PowerShell fallback — enumerate Win32_SoundDevice
      const ps = Bun.spawn(
        ["powershell", "-NoProfile", "-Command",
          "Get-CimInstance Win32_SoundDevice | Select-Object -ExpandProperty Caption"],
        { stdout: "pipe", stderr: "pipe" },
      )
      const psOut = await new Response(ps.stdout).text()
      await ps.exited
      const names = psOut.split("\n").map(s => s.trim()).filter(Boolean)
      const fallback: AudioDevice[] = names.map((name, i) => ({
        index: i + 1,
        name,
        isDefault: i === 0,
        sampleRate: 48000,
        channels: 2,
      }))
      if (fallback.length > 0) return { devices: fallback, error: null }
    }
    return { devices, error: null }
  } catch (e) {
    return { devices: [], error: e instanceof Error ? e.message : String(e) }
  }
}

/** @deprecated Use listDevicesWithError() for proper error handling. */
export async function listDevices(): Promise<AudioDevice[]> {
  const { devices } = await listDevicesWithError()
  return devices
}

// ── Model listing ────────────────────────────────────────────────────────

/** Hardcoded model registry (mirrors src/models/registry.rs) */
export function getModelRegistry(): ModelInfo[] {
  return [
    {
      id: "ggml-tiny.en",
      name: "Whisper Tiny English (GGML)",
      size: "~75 MB",
      sizeBytes: 75_000_000,
      speedFactor: 32,
      memoryMb: 200,
      description: "Fastest model using whisper.cpp",
      downloaded: false,
    },
    {
      id: "ggml-base.en",
      name: "Whisper Base English (GGML)",
      size: "~140 MB",
      sizeBytes: 140_000_000,
      speedFactor: 16,
      memoryMb: 300,
      description: "Best balance of speed and accuracy",
      downloaded: false,
    },
    {
      id: "ggml-small.en",
      name: "Whisper Small English (GGML)",
      size: "~470 MB",
      sizeBytes: 470_000_000,
      speedFactor: 8,
      memoryMb: 600,
      description: "Higher accuracy, still suitable for dictation",
      downloaded: false,
    },
  ]
}

export async function listModelsWithStatus(): Promise<ModelInfo[]> {
  const models = getModelRegistry()
  try {
    const out = await run(["models", "downloaded"])
    const downloadedIds = out
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.length > 0)
    for (const m of models) {
      if (downloadedIds.some((d) => d.includes(m.id))) {
        m.downloaded = true
      }
    }
  } catch {
    // Ignore — will show all as not downloaded
  }
  return models
}

export async function downloadModel(modelId: string): Promise<string> {
  return run(["models", "download", modelId])
}

export async function isModelDownloaded(modelId: string): Promise<boolean> {
  try {
    const output = await run(["models", "downloaded"])
    return output.includes(modelId)
  } catch {
    return false
  }
}

export async function removeModel(modelId: string): Promise<string> {
  return run(["models", "remove", modelId])
}

// ── Daemon status ────────────────────────────────────────────────────────

export async function getDaemonStatus(): Promise<DaemonStatus | null> {
  try {
    const out = await run(["status"])
    // Best-effort parse of status output
    return {
      version: "0.1.0",
      state: out.includes("running") ? "Running" : "Stopped",
      modelLoaded: out.includes("model loaded"),
      modelName: null,
      isDictating: out.includes("dictating"),
    }
  } catch {
    return null
  }
}

// ── Daemon control ───────────────────────────────────────────────────────

export async function reloadDaemonConfig(): Promise<ReloadResult> {
  try {
    // First reload the config in the daemon
    await run(["reload-config"])
    
    // Then restart the daemon to apply changes
    const platform = process.platform
    
    // macOS with launchd
    if (platform === "darwin") {
      try {
        const label = "com.onevox.daemon"
        const uidProc = Bun.spawn(["id", "-u"], { stdout: "pipe", stderr: "pipe" })
        const uid = (await new Response(uidProc.stdout).text()).trim()
        await uidProc.exited
        
        const kickstartProc = Bun.spawn(["launchctl", "kickstart", "-k", `gui/${uid}/${label}`], {
          stdout: "pipe",
          stderr: "pipe",
        })
        await kickstartProc.exited
        
        // Wait for daemon to restart
        await new Promise(resolve => setTimeout(resolve, 1500))
        
        return {
          state: "reloaded",
          message: "Configuration reloaded and daemon restarted",
        }
      } catch (restartError) {
        // Fallback: try stop/start via CLI
        try {
          await run(["stop"])
          await new Promise(resolve => setTimeout(resolve, 500))
          // Daemon should auto-restart via launchd
          await new Promise(resolve => setTimeout(resolve, 1500))
          
          return {
            state: "reloaded",
            message: "Configuration reloaded and daemon restarted",
          }
        } catch {
          return {
            state: "reloaded",
            message: "Config reloaded - restart manually: launchctl kickstart -k gui/$(id -u)/com.onevox.daemon",
          }
        }
      }
    }
    
    // Linux with systemd
    if (platform === "linux") {
      try {
        const restartProc = Bun.spawn(["systemctl", "--user", "restart", "onevox"], {
          stdout: "pipe",
          stderr: "pipe",
        })
        await restartProc.exited
        
        // Wait for daemon to restart
        await new Promise(resolve => setTimeout(resolve, 1500))
        
        return {
          state: "reloaded",
          message: "Configuration reloaded and daemon restarted",
        }
      } catch (systemdError) {
        // Fallback: try stop/start via CLI (for non-systemd Linux)
        try {
          await run(["stop"])
          await new Promise(resolve => setTimeout(resolve, 500))
          // User needs to manually start daemon
          return {
            state: "reloaded",
            message: "Config reloaded - start daemon manually: onevox daemon",
          }
        } catch {
          return {
            state: "reloaded",
            message: "Config reloaded - restart manually: systemctl --user restart onevox",
          }
        }
      }
    }
    
    // Windows (no service manager, use CLI stop/start)
    if (platform === "win32") {
      try {
        await run(["stop"])
        await new Promise(resolve => setTimeout(resolve, 500))
        
        // On Windows, user typically runs daemon manually or via Task Scheduler
        return {
          state: "reloaded",
          message: "Config reloaded - start daemon manually: onevox daemon",
        }
      } catch {
        return {
          state: "reloaded",
          message: "Config reloaded - restart daemon manually: onevox stop && onevox daemon",
        }
      }
    }
    
    // Fallback for other platforms
    return {
      state: "reloaded",
      message: "Config reloaded - restart daemon manually: onevox stop && onevox daemon",
    }
    
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e)
    // Check if daemon is not running
    if (errorMsg.includes("not running") || errorMsg.includes("Failed to connect")) {
      return {
        state: "not_running",
        message: errorMsg,
      }
    }
    return {
      state: "failed",
      message: errorMsg,
    }
  }
}
