/**
 * ONEVOX TUI — Entry point
 *
 * Creates the OpenTUI renderer and boots the app.
 */

import { createCliRenderer } from "@opentui/core"
import { loadConfig } from "./data/config.js"
import { loadHistory } from "./data/history.js"
import { createApp } from "./app.js"

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
    process.exit(0)
  })
}

main().catch((err) => {
  console.error("ONEVOX TUI failed to start:", err)
  process.exit(1)
})
