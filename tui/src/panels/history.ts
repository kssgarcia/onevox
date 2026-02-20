/**
 * History Panel — scrollable list of transcription history entry cards.
 *
 * Features:
 *   - Newest-first card list in a ScrollBox
 *   - Per-card actions: Copy, Export, Expand, Delete
 *   - Full-text expansion in a popup overlay
 *   - Keyboard nav: Up/Down, c=copy, e=export, Enter=expand, d=delete, D=clear
 */

import {
  BoxRenderable,
  TextRenderable,
  ScrollBoxRenderable,
  type CliRenderer,
  RGBA,
  TextAttributes,
} from "@opentui/core"

import type { AppState } from "../app.js"
import type { HistoryEntry } from "../data/history.js"
import {
  newestFirst,
  formatTimestamp,
  formatDuration,
  truncateText,
  removeEntry,
  clearHistory,
  saveHistory,
} from "../data/history.js"
import { createCard, type CardInstance } from "../components/card.js"
import { createConfirmPopup } from "../components/confirm-popup.js"

export interface HistoryPanelCallbacks {
  onStatusMessage: (msg: string) => void
}

export interface HistoryPanelInstance {
  root: BoxRenderable
  refresh: () => void
}

export function createHistoryPanel(
  renderer: CliRenderer,
  state: AppState,
  callbacks: HistoryPanelCallbacks,
): HistoryPanelInstance {
  let selectedIndex = 0
  let cards: CardInstance[] = []
  let expandedPopup: BoxRenderable | null = null
  
  const theme = state.theme

  // ── Root ─────────────────────────────────────────────────────────────
  const root = new BoxRenderable(renderer, {
    id: "history-panel",
    width: "100%" as any,
    height: "100%" as any,
    flexDirection: "column",
  })

  // ── Top bar ──────────────────────────────────────────────────────────
  const topBar = new BoxRenderable(renderer, {
    id: "history-topbar",
    width: "100%" as any,
    height: 1,
    flexDirection: "row",
    justifyContent: "space-between",
    marginBottom: 2,
    paddingBottom: 1,
  })

  const titleText = new TextRenderable(renderer, {
    id: "history-title",
    content: "Transcription History",
    fg: theme.colors.textPrimary,
    attributes: TextAttributes.BOLD,
  })

  const countText = new TextRenderable(renderer, {
    id: "history-count",
    content: "",
    fg: theme.colors.textSecondary,
  })

  topBar.add(titleText)
  topBar.add(countText)

  // ── Scroll container ─────────────────────────────────────────────────
  const scrollBox = new ScrollBoxRenderable(renderer, {
    id: "history-scroll",
    width: "100%" as any,
    height: "100%" as any,
    viewportCulling: true,
  })

  root.add(topBar)
  root.add(scrollBox)

  // ── Build cards ──────────────────────────────────────────────────────

  function buildCards() {
    // Clear existing
    cards = []
    try {
      const children = (scrollBox as any).content?.children
      if (children) {
        while (children.length > 0) {
          scrollBox.remove(children[0].id)
        }
      }
    } catch {}

    const entries = newestFirst(state.history)
    countText.content = `${entries.length} entries`

    if (entries.length === 0) {
      const emptyBox = new BoxRenderable(renderer, {
        id: "history-empty-box",
        width: "100%" as any,
        flexDirection: "column",
        justifyContent: "center",
        alignItems: "center",
        paddingTop: 4,
        gap: 1,
      })
      
      const emptyIcon = new TextRenderable(renderer, {
        id: "history-empty-icon",
        content: "🎤",
        fg: theme.colors.inactive,
      })
      
      const emptyText = new TextRenderable(renderer, {
        id: "history-empty",
        content: "No transcription history yet",
        fg: theme.colors.textMuted,
      })
      
      const emptyHint = new TextRenderable(renderer, {
        id: "history-empty-hint",
        content: "Start dictating to see entries here",
        fg: theme.colors.textMuted,
      })
      
      emptyBox.add(emptyIcon)
      emptyBox.add(emptyText)
      emptyBox.add(emptyHint)
      scrollBox.add(emptyBox)
      return
    }

    for (let i = 0; i < entries.length; i++) {
      const entry = entries[i]
      const subtitle = `${entry.model} • ${formatTimestamp(entry.timestamp)} • ${formatDuration(entry.duration_ms)}${entry.confidence != null ? ` • ${(entry.confidence * 100).toFixed(0)}%` : ""}`

      const card = createCard(renderer, {
        id: `hist-${entry.id}`,
        text: truncateText(entry.text, 80),
        subtitle,
        selected: i === selectedIndex,
        theme,
        actions: [
          {
            label: "Copy",
            handler: () => copyEntry(entry),
          },
          {
            label: "Export",
            handler: () => exportEntry(entry),
          },
          {
            label: "Expand",
            handler: () => expandEntry(entry),
          },
          {
            label: "Delete",
            handler: () => deleteEntry(entry),
          },
        ],
      })

      // Make the entire card clickable to select it
      card.root.onMouseDown = () => {
        selectedIndex = i
        updateSelection()
      }

      cards.push(card)
      scrollBox.add(card.root)
    }
  }

  function updateSelection() {
    for (let i = 0; i < cards.length; i++) {
      cards[i].setSelected(i === selectedIndex)
    }
    
    // Smooth scroll to selected card
    // Each card is roughly 4 lines tall (title + subtitle + padding + margin)
    const cardHeight = 4
    const selectedTop = selectedIndex * cardHeight
    const viewportHeight = (scrollBox as any).viewportHeight || 20
    const currentScroll = scrollBox.scrollTop
    
    // Scroll to keep selected card in view
    if (selectedTop < currentScroll) {
      scrollBox.scrollTop = selectedTop
    } else if (selectedTop + cardHeight > currentScroll + viewportHeight) {
      scrollBox.scrollTop = Math.max(0, selectedTop + cardHeight - viewportHeight)
    }
  }

  // ── Actions ──────────────────────────────────────────────────────────

  function getSelectedEntry(): HistoryEntry | null {
    const entries = newestFirst(state.history)
    return entries[selectedIndex] ?? null
  }

  function copyEntry(entry: HistoryEntry) {
    // Use clipboard via platform command
    try {
      const cmd =
        process.platform === "win32"
          ? ["cmd", "/c", `echo ${entry.text.replace(/"/g, '\\"')} | clip`]
          : process.platform === "darwin"
            ? ["pbcopy"]
            : ["xclip", "-selection", "clipboard"]

      if (process.platform === "win32") {
        Bun.spawn(["powershell", "-Command", `Set-Clipboard -Value '${entry.text.replace(/'/g, "''")}'`])
      } else {
        const proc = Bun.spawn(cmd, { stdin: "pipe" })
        proc.stdin.write(entry.text)
        proc.stdin.end()
      }
      callbacks.onStatusMessage("✓ Copied to clipboard")
    } catch {
      callbacks.onStatusMessage("✗ Failed to copy")
    }
    setTimeout(() => callbacks.onStatusMessage(""), 2000)
  }

  function exportEntry(entry: HistoryEntry) {
    try {
      const { existsSync, mkdirSync, appendFileSync } = require("node:fs")
      const { join } = require("node:path")
      const { homedir } = require("node:os")
      const exportDir = join(homedir(), "onevox-exports")
      if (!existsSync(exportDir)) mkdirSync(exportDir, { recursive: true })
      const exportFile = join(exportDir, "transcriptions.txt")
      const line = `[${formatTimestamp(entry.timestamp)}] (${entry.model}) ${entry.text}\n`
      appendFileSync(exportFile, line, "utf-8")
      callbacks.onStatusMessage(`✓ Exported to ~/onevox-exports/transcriptions.txt`)
    } catch {
      callbacks.onStatusMessage("✗ Failed to export")
    }
    setTimeout(() => callbacks.onStatusMessage(""), 3000)
  }

  function expandEntry(entry: HistoryEntry) {
    if (expandedPopup) {
      try { renderer.root.remove("expand-overlay") } catch {}
      expandedPopup = null
      return
    }

    const overlay = new BoxRenderable(renderer, {
      id: "expand-overlay",
      position: "absolute" as any,
      width: "100%" as any,
      height: "100%" as any,
      justifyContent: "center",
      alignItems: "center",
      backgroundColor: RGBA.fromInts(0, 0, 0, 100),
    })

    const popup = new BoxRenderable(renderer, {
      id: "expand-popup",
      width: "80%" as any,
      height: "60%" as any,
      backgroundColor: RGBA.fromHex(theme.colors.bg),
      padding: 3,
      flexDirection: "column",
      gap: 2,
      title: "Full Transcription",
      titleAlignment: "center" as any,
    })

    const scrollableText = new ScrollBoxRenderable(renderer, {
      id: "expand-scroll",
      width: "100%" as any,
      flexGrow: 1,
    })

    const fullText = new TextRenderable(renderer, {
      id: "expand-text",
      content: entry.text,
      fg: theme.colors.textPrimary,
    })

    scrollableText.add(fullText)

    const metaText = new TextRenderable(renderer, {
      id: "expand-meta",
      content: `Model: ${entry.model}  │  ${formatTimestamp(entry.timestamp)}  │  Duration: ${formatDuration(entry.duration_ms)}`,
      fg: theme.colors.textSecondary,
    })

    const closeHint = new TextRenderable(renderer, {
      id: "expand-hint",
      content: "Press Esc or Enter to close",
      fg: theme.colors.textMuted,
    })

    popup.add(scrollableText)
    popup.add(metaText)
    popup.add(closeHint)
    overlay.add(popup)

    expandedPopup = overlay
    renderer.root.add(overlay)

    // Close handler
    const closeHandler = (seq: string) => {
      if (seq === "\x1b" || seq === "\r") {
        try { renderer.root.remove("expand-overlay") } catch {}
        expandedPopup = null
        renderer.removeInputHandler?.(closeHandler)
        return true
      }
      return true // consume all input
    }
    renderer.prependInputHandler(closeHandler)
  }

  function deleteEntry(entry: HistoryEntry) {
    const popup = createConfirmPopup(renderer, {
      id: "delete-confirm",
      title: "Delete Entry",
      message: `Delete this transcription?\n"${truncateText(entry.text, 50)}"`,
      theme,
      onConfirm: () => {
        state.history = removeEntry(state.history, entry.id)
        saveHistory(state.history)
        if (selectedIndex >= newestFirst(state.history).length) {
          selectedIndex = Math.max(0, newestFirst(state.history).length - 1)
        }
        buildCards()
        callbacks.onStatusMessage("✓ Entry deleted")
        setTimeout(() => callbacks.onStatusMessage(""), 2000)
      },
      onCancel: () => {},
    })
    renderer.root.add(popup.root)
  }

  function clearAll() {
    const popup = createConfirmPopup(renderer, {
      id: "clear-confirm",
      title: "Clear All",
      message: `Delete all ${state.history.length} transcription entries?\nThis cannot be undone.`,
      theme,
      onConfirm: () => {
        state.history = clearHistory()
        saveHistory(state.history)
        selectedIndex = 0
        buildCards()
        callbacks.onStatusMessage("✓ All entries cleared")
        setTimeout(() => callbacks.onStatusMessage(""), 2000)
      },
      onCancel: () => {},
    })
    renderer.root.add(popup.root)
  }

  // ── Keyboard handling ────────────────────────────────────────────────

  renderer.keyInput.on("keypress", (key: any) => {
    // Only respond when history tab is active
    if (state.activeTab !== 0) return
    if (expandedPopup) return // Let popup handle input

    const entries = newestFirst(state.history)
    if (entries.length === 0) return

    if (key.name === "down" || key.name === "j") {
      selectedIndex = Math.min(selectedIndex + 1, entries.length - 1)
      updateSelection()
      return
    }
    if (key.name === "up" || key.name === "k") {
      selectedIndex = Math.max(selectedIndex - 1, 0)
      updateSelection()
      return
    }
    if (key.name === "c" && !key.ctrl) {
      const entry = getSelectedEntry()
      if (entry) copyEntry(entry)
      return
    }
    if (key.name === "e" && !key.ctrl) {
      const entry = getSelectedEntry()
      if (entry) exportEntry(entry)
      return
    }
    if (key.name === "return") {
      const entry = getSelectedEntry()
      if (entry) expandEntry(entry)
      return
    }
    if (key.name === "d" && !key.shift) {
      const entry = getSelectedEntry()
      if (entry) deleteEntry(entry)
      return
    }
    if (key.name === "d" && key.shift) {
      clearAll()
      return
    }
  })

  // ── Initial render ───────────────────────────────────────────────────
  buildCards()

  return {
    root,
    refresh: buildCards,
  }
}
