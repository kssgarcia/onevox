/**
 * Help Overlay — keyboard shortcut reference popup.
 */

import {
  BoxRenderable,
  TextRenderable,
  type CliRenderer,
  RGBA,
  TextAttributes,
} from "@opentui/core"

import type { Theme } from "../theme.js"

export interface HelpOverlayInstance {
  root: BoxRenderable
  destroy: () => void
}

export function createHelpOverlay(
  renderer: CliRenderer,
  onClose: () => void,
  theme: Theme,
): HelpOverlayInstance {
  const overlay = new BoxRenderable(renderer, {
    id: "help-overlay",
    position: "absolute" as any,
    width: "100%" as any,
    height: "100%" as any,
    justifyContent: "center",
    alignItems: "center",
    backgroundColor: RGBA.fromInts(0, 0, 0, 80),
  })

  const popup = new BoxRenderable(renderer, {
    id: "help-popup",
    width: 65,
    backgroundColor: RGBA.fromHex(theme.colors.bg),
    padding: 2,
    flexDirection: "column",
    gap: 1,
    title: "Keyboard Shortcuts",
    titleAlignment: "center" as any,
  })

  const sections = [
    {
      title: "Navigation",
      keys: [
        ["← →  or  h l", "Switch between History / Config tabs"],
        ["↓  or  j", "Enter content area / Move down"],
        ["↑  or  k", "Move up"],
        ["Enter", "Select / Activate item"],
        ["Esc", "Return to tab bar"],
      ],
    },
    {
      title: "Global",
      keys: [
        ["t", "Toggle dark/light theme"],
        ["?", "Toggle this help overlay"],
        ["Ctrl+S", "Save config changes"],
        ["Ctrl+C  or  q", "Quit ONEVOX"],
      ],
    },
    {
      title: "History Tab",
      keys: [
        ["↑ ↓  or  k j", "Navigate entries"],
        ["Enter", "Expand full transcription text"],
        ["c", "Copy selected entry to clipboard"],
        ["e", "Export selected entry to file"],
        ["dd  or  x", "Delete selected entry (Vim-style)"],
        ["D (Shift+d)", "Clear all history"],
      ],
    },
    {
      title: "Config Tab",
      keys: [
        ["↑ ↓  or  k j", "Navigate between fields"],
        ["← →  or  h l", "Change stepper values"],
        ["Space", "Toggle switches"],
        ["Enter", "Activate dropdown/select"],
      ],
    },
    {
      title: "Popups & Dialogs",
      keys: [
        ["y", "Confirm action"],
        ["n  or  Esc", "Cancel / close popup"],
        ["Enter", "Confirm / close"],
      ],
    },
    {
      title: "Model Backends",
      keys: [
        ["whisper_cpp", "OpenAI Whisper (default, CPU/GPU)"],
        ["onnx", "ONNX Runtime - Parakeet (15-25x RT, CPU)"],
      ],
    },
  ]

  for (const section of sections) {
    const sectionTitle = new TextRenderable(renderer, {
      id: `help-sec-${section.title}`,
      content: section.title,
      fg: theme.colors.textPrimary,
      attributes: TextAttributes.BOLD,
    })
    popup.add(sectionTitle)

    for (const [key, desc] of section.keys) {
      const row = new BoxRenderable(renderer, {
        id: `help-key-${key.replace(/[\s\/]+/g, "-")}`,
        flexDirection: "row",
        height: 1,
        paddingLeft: 2,
      })

      const keyText = new TextRenderable(renderer, {
        id: `help-k-${key.replace(/[\s\/]+/g, "-")}`,
        content: key.padEnd(18),
        fg: theme.colors.textPrimary,
        attributes: TextAttributes.BOLD,
      })

      const descText = new TextRenderable(renderer, {
        id: `help-d-${key.replace(/[\s\/]+/g, "-")}`,
        content: desc,
        fg: theme.colors.textSecondary,
      })

      row.add(keyText)
      row.add(descText)
      popup.add(row)
    }
  }

  // Close hint
  const closeHint = new TextRenderable(renderer, {
    id: "help-close-hint",
    content: "\n  Press ? or Esc to close",
    fg: theme.colors.textMuted,
  })
  popup.add(closeHint)

  overlay.add(popup)

  // ── Input handler ──────────────────────────────────────────────────
  const handler = (sequence: string) => {
    if (sequence === "?" || sequence === "\x1b") {
      destroy()
      return true
    }
    return true // consume all input while help is open
  }
  renderer.prependInputHandler(handler)

  function destroy() {
    renderer.removeInputHandler?.(handler)
    try {
      renderer.root.remove("help-overlay")
    } catch {}
    onClose()
  }

  return { root: overlay, destroy }
}
