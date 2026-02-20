/**
 * App — root layout with ASCII header, TabSelect, content area, status bar.
 *
 * ┌─────────────────────────────────────────────┐
 * │  █▀█ █▄ █ █▀▀ █ █ █▀█ ▀▄▀   (ASCIIFont)   │
 * │  History    Config                (tabs)     │
 * │ ┌─────────────────────────────────────────┐  │
 * │ │  Content area (History or Config panel) │  │
 * │ └─────────────────────────────────────────┘  │
 * │  Tab: switch tabs | ?: help | q: quit        │
 * └─────────────────────────────────────────────┘
 */

import {
  BoxRenderable,
  TextRenderable,
  TabSelectRenderable,
  TabSelectRenderableEvents,
  ASCIIFontRenderable,
  type CliRenderer,
  RGBA,
} from "@opentui/core"

import type { VoxConfig } from "./data/config.js"
import type { HistoryEntry } from "./data/history.js"
import { saveConfig } from "./data/config.js"
import { getTheme, type Theme, type ThemeName } from "./theme.js"
import { createHistoryPanel, type HistoryPanelInstance } from "./panels/history.js"
import { createConfigPanel, type ConfigPanelInstance } from "./panels/config.js"
import { createHelpOverlay, type HelpOverlayInstance } from "./panels/help.js"

export interface AppState {
  config: VoxConfig
  history: HistoryEntry[]
  configDirty: boolean
  activeTab: number // 0 = History, 1 = Config
  theme: Theme
}

export interface AppInstance {
  state: AppState
  destroy: () => void
}

export function createApp(
  renderer: CliRenderer,
  config: VoxConfig,
  history: HistoryEntry[],
): AppInstance {
  // Load theme from config or default to dark
  const themeName: ThemeName = config.ui?.theme || "dark"
  const initialTheme = getTheme(themeName)
  
  const state: AppState = {
    config,
    history,
    configDirty: false,
    activeTab: 0,
    theme: initialTheme,
  }

  let helpOverlay: HelpOverlayInstance | null = null
  let historyPanel: HistoryPanelInstance | null = null
  let configPanel: ConfigPanelInstance | null = null

  // ── Root container ────────────────────────────────────────────────────
  const root = new BoxRenderable(renderer, {
    id: "app-root",
    width: "100%" as any,
    height: "100%" as any,
    flexDirection: "column",
    backgroundColor: RGBA.fromHex(state.theme.colors.bg),
  })

  // ── Header ────────────────────────────────────────────────────────────
  const headerBox = new BoxRenderable(renderer, {
    id: "header",
    width: "100%" as any,
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    paddingLeft: 2,
    paddingRight: 2,
    paddingTop: 1,
    paddingBottom: 2,
  })

  const asciiTitle = new ASCIIFontRenderable(renderer, {
    id: "ascii-title",
    text: "ONEVOX",
    font: "block" as any,
    color: [RGBA.fromHex(state.theme.colors.accent), RGBA.fromHex(state.theme.colors.textSecondary)] as any,
  })

  const versionText = new TextRenderable(renderer, {
    id: "version",
    content: "v0.1.0",
    fg: state.theme.colors.textMuted,
  })

  headerBox.add(asciiTitle)
  headerBox.add(versionText)

  // ── Tabs ──────────────────────────────────────────────────────────────
  const tabBar = new BoxRenderable(renderer, {
    id: "tab-bar",
    width: "100%" as any,
    paddingLeft: 2,
    paddingRight: 2,
    paddingBottom: 1,
    height: 3,
  })

  const tabs = new TabSelectRenderable(renderer, {
    id: "tabs",
    width: "100%" as any,
    options: [
      { name: "History", description: "Transcription history" },
      { name: "Config", description: "Settings and configuration" },
    ],
    tabWidth: 20,
    backgroundColor: RGBA.fromHex(state.theme.colors.bg),
    textColor: RGBA.fromHex(state.theme.colors.textMuted),
    selectedBackgroundColor: RGBA.fromHex(state.theme.colors.bg),
    selectedTextColor: RGBA.fromHex(state.theme.colors.textPrimary),
    showDescription: false,
    showUnderline: true,
  })

  // Enable mouse interaction for tabs
  tabs.onMouseDown = (event: any) => {
    setFocusMode("tabs")
    tabs.focus()
  }

  tabBar.add(tabs)

  // ── Content area ──────────────────────────────────────────────────────
  const contentArea = new BoxRenderable(renderer, {
    id: "content-area",
    width: "100%" as any,
    flexGrow: 1,
    paddingTop: 2,
    paddingBottom: 2,
    paddingLeft: 2,
    paddingRight: 2,
  })

  // ── Status bar ────────────────────────────────────────────────────────
  const statusBar = new BoxRenderable(renderer, {
    id: "status-bar",
    width: "100%" as any,
    height: 1,
    flexDirection: "row",
    justifyContent: "space-between",
    paddingLeft: 2,
    paddingRight: 2,
    backgroundColor: RGBA.fromHex(state.theme.colors.statusBar),
  })

  const statusLeft = new TextRenderable(renderer, {
    id: "status-left",
    content: "",
    fg: state.theme.colors.textPrimary,
  })

  const statusCenter = new TextRenderable(renderer, {
    id: "status-center",
    content: "",
    fg: state.theme.colors.textPrimary,
  })

  const statusRight = new TextRenderable(renderer, {
    id: "status-right",
    content: `● ${state.theme.name === "dark" ? "Dark" : "Light"} Mode`,
    fg: state.theme.colors.textSecondary,
  })

  statusBar.add(statusLeft)
  statusBar.add(statusCenter)
  statusBar.add(statusRight)

  // ── Assemble layout ───────────────────────────────────────────────────
  root.add(headerBox)
  root.add(tabBar)
  root.add(contentArea)
  root.add(statusBar)

  renderer.root.add(root)
  tabs.focus()

  // ── Panel management ──────────────────────────────────────────────────

  // Focus state: "tabs" | "content"
  let focusMode: "tabs" | "content" = "tabs"

  function refreshStatusHints() {
    const general = "Esc Top  Enter Open  ↑/↓ j/k Move  ? Help  t Theme  Ctrl+C Quit"
    if (focusMode === "tabs") {
      statusLeft.content = `${general}  |  ←/→ h/l Tabs`
    } else {
      statusLeft.content = `${general}  |  Esc -> Tabs`
    }
    statusRight.content = `● ${state.theme.name === "dark" ? "Dark" : "Light"} Mode`
  }

  function applyFocusModeStyles() {
    const tabsFocused = focusMode === "tabs"
    tabBar.backgroundColor = RGBA.fromHex(state.theme.colors.bg)
    tabs.backgroundColor = RGBA.fromHex(state.theme.colors.bg)
    tabs.textColor = RGBA.fromHex(state.theme.colors.textMuted)
    tabs.selectedBackgroundColor = RGBA.fromHex(tabsFocused ? state.theme.colors.selected : state.theme.colors.bg)
    tabs.selectedTextColor = RGBA.fromHex(state.theme.colors.textPrimary)
  }

  function setFocusMode(mode: "tabs" | "content") {
    focusMode = mode
    applyFocusModeStyles()
    refreshStatusHints()
  }
  applyFocusModeStyles()
  refreshStatusHints()

  function showHistory() {
    clearContent()
    historyPanel = createHistoryPanel(renderer, state, {
      onStatusMessage: (msg) => { statusCenter.content = msg },
      onEscape: () => {
        // Return focus to tabs
        setFocusMode("tabs")
        tabs.focus()
      },
    })
    contentArea.add(historyPanel.root)
  }

  function showConfig() {
    clearContent()
    configPanel = createConfigPanel(renderer, state, {
      onDirty: () => {
        state.configDirty = true
        statusCenter.fg = RGBA.fromHex(state.theme.colors.textSecondary)
        statusCenter.content = "● Unsaved changes (Ctrl+S to save)"
      },
      onSaved: () => {
        state.configDirty = false
        statusCenter.content = "✓ Saved"
        statusCenter.fg = RGBA.fromHex(state.theme.colors.textPrimary)
        setTimeout(() => {
          if (!state.configDirty) {
            statusCenter.content = ""
            statusCenter.fg = RGBA.fromHex(state.theme.colors.textPrimary)
          }
        }, 3000)
      },
      onStatusMessage: (msg) => {
        statusCenter.fg = RGBA.fromHex(state.theme.colors.textPrimary)
        statusCenter.content = msg
      },
      onEscape: () => {
        // Return focus to tabs
        setFocusMode("tabs")
        tabs.focus()
      },
    })
    configPanel.root.onMouseDown = () => {
      setFocusMode("content")
    }
    contentArea.add(configPanel.root)
  }

  function clearContent() {
    if (historyPanel) {
      try { contentArea.remove("history-panel") } catch {}
      historyPanel = null
    }
    if (configPanel) {
      configPanel.destroy()
      try { contentArea.remove("config-panel") } catch {}
      configPanel = null
    }
  }

  // ── Tab switching ─────────────────────────────────────────────────────

  tabs.on(TabSelectRenderableEvents.SELECTION_CHANGED, (index: number) => {
    state.activeTab = index
    setFocusMode("content")
    if (index === 0) {
      showHistory()
      if (historyPanel) historyPanel.focusFirst?.()
    } else {
      showConfig()
      if (configPanel) configPanel.focusFirst()
    }
  })

  // Show initial tab
  showHistory()

  // ── Theme toggle ──────────────────────────────────────────────────────
  
  function toggleTheme() {
    // Toggle theme name
    const newThemeName: ThemeName = state.theme.name === "dark" ? "light" : "dark"
    state.theme = getTheme(newThemeName)
    
    // Save to config
    if (!state.config.ui) state.config.ui = { theme: newThemeName }
    else state.config.ui.theme = newThemeName
    saveConfig(state.config)
    
    // Update all UI elements
    root.backgroundColor = RGBA.fromHex(state.theme.colors.bg)
    asciiTitle.color = [RGBA.fromHex(state.theme.colors.accent), RGBA.fromHex(state.theme.colors.textSecondary)] as any
    versionText.fg = RGBA.fromHex(state.theme.colors.textMuted)
    
    applyFocusModeStyles()

    statusBar.backgroundColor = RGBA.fromHex(state.theme.colors.statusBar)
    statusLeft.fg = RGBA.fromHex(state.theme.colors.textPrimary)
    statusCenter.fg = RGBA.fromHex(state.theme.colors.textPrimary)
    statusRight.fg = RGBA.fromHex(state.theme.colors.textSecondary)
    refreshStatusHints()
    
    // Rebuild current panel with new theme
    if (state.activeTab === 0) showHistory()
    else showConfig()
    
    // Show feedback
    statusCenter.content = `✓ Switched to ${state.theme.name} mode`
    statusCenter.fg = RGBA.fromHex(state.theme.colors.textPrimary)
    setTimeout(() => {
      if (!state.configDirty) {
        statusCenter.content = ""
      }
    }, 2000)
  }

  // ── Global keyboard handler ───────────────────────────────────────────

  renderer.keyInput.on("keypress", (key: any) => {
    // q to quit (only if no popup is open)
    if (key.name === "q" && !key.ctrl && !key.meta && !helpOverlay) {
      renderer.destroy()
      return
    }

    // t to toggle theme
    if (key.name === "t" && !key.ctrl && !key.meta && !helpOverlay) {
      toggleTheme()
      return
    }

    // ? for help
    if (key.shift && key.name === "/" || key.name === "?") {
      if (helpOverlay) {
        helpOverlay.destroy()
        helpOverlay = null
      } else {
        helpOverlay = createHelpOverlay(renderer, () => {
          helpOverlay = null
        }, state.theme)
        renderer.root.add(helpOverlay.root)
      }
      return
    }

    // Ctrl+S to save config
    if (key.ctrl && key.name === "s") {
      if (state.configDirty && configPanel) {
        configPanel.save()
      } else if (!state.configDirty && state.activeTab === 1) {
        statusCenter.content = "Nothing to save"
        statusCenter.fg = RGBA.fromHex(state.theme.colors.textMuted)
        setTimeout(() => {
          statusCenter.content = ""
          statusCenter.fg = RGBA.fromHex(state.theme.colors.textPrimary)
        }, 1500)
      }
      return
    }

    // Left/Right or h/l: Navigate between tabs (only when tabs are focused)
    if (focusMode === "tabs") {
      if (key.name === "left" || key.name === "right" || key.name === "h" || key.name === "l") {
        const next = (state.activeTab + 1) % 2
        tabs.setSelectedIndex(next)
        state.activeTab = next
        return
      }
      
      // Enter or Down or j: Enter content area
      if (key.name === "return" || key.name === "down" || key.name === "j") {
        setFocusMode("content")
        if (state.activeTab === 0) {
          if (historyPanel) historyPanel.focusFirst?.()
        } else {
          if (configPanel) configPanel.focusFirst()
        }
        return
      }
    }
    
    // Escape: Return to tabs from content
    if (key.name === "escape" && focusMode === "content") {
      if (state.activeTab === 0) {
        if (historyPanel) historyPanel.blurAll?.()
      } else {
        if (configPanel && configPanel.hasFocus()) {
          configPanel.blurAll()
        }
      }
      setFocusMode("tabs")
      tabs.focus()
      return
    }
  })

  function destroy() {
    renderer.destroy()
  }

  return { state, destroy }
}
