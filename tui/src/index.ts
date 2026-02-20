/**
 * ONEVOX TUI — Entry point
 *
 * Creates the OpenTUI renderer and boots the app.
 */

import { createCliRenderer } from "@opentui/core"
import { writeSync } from "node:fs"
import { loadConfig } from "./data/config.js"
import { loadHistory } from "./data/history.js"
import { createApp } from "./app.js"

function restoreTerminalState() {
  // Ensure stdin isn't left in raw mode.
  try {
    if (process.stdin.isTTY) {
      process.stdin.setRawMode(false)
    }
  } catch {}
  // Disable mouse tracking modes and alt screen, show cursor, reset styles.
  const reset = "\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1015l\x1b[?1049l\x1b[?25h\x1b[0m"
  try { writeSync(1, reset) } catch {}
  try { writeSync(2, reset) } catch {}
}

async function main() {
  // Load data from disk
  const config = loadConfig()
  const history = loadHistory()

  // Create the OpenTUI renderer
  const renderer = await createCliRenderer({
    exitOnCtrlC: true,
    targetFps: 30,
    useMouse: true,
    useAlternateScreen: true,
  })

  // Boot the application
  const app = createApp(renderer, config, history)
  let exiting = false
  let exitCode = 0

  const finalizeExit = (code: number) => {
    if (exiting) return
    exiting = true
    restoreTerminalState()
    process.exit(code)
  }

  const exitCleanly = (code: number) => {
    exitCode = code
    try {
      renderer.destroy()
    } catch {
      finalizeExit(code)
    }
  }

  process.on("SIGINT", () => exitCleanly(130))
  process.on("SIGTERM", () => exitCleanly(143))
  process.on("uncaughtException", () => exitCleanly(1))
  process.on("exit", restoreTerminalState)

  // Handle clean exit
  renderer.on("destroy", () => {
    // Save if dirty
    if (app.state.configDirty) {
      try {
        const { saveConfig } = require("./data/config.js")
        saveConfig(app.state.config)
      } catch {
        // Best effort
      }
    }
    finalizeExit(exitCode)
  })
}

main().catch((err) => {
  console.error("ONEVOX TUI failed to start:", err)
  process.exit(1)
})
