/**
 * ConfirmPopup — a modal confirmation dialog centered on screen.
 *
 * Usage:
 *   const popup = createConfirmPopup(renderer, {
 *     message: "Delete all entries?",
 *     onConfirm: () => { ... },
 *     onCancel: () => { ... },
 *   })
 *   renderer.root.add(popup.root)
 *   popup.focus()
 */

import {
  BoxRenderable,
  TextRenderable,
  type CliRenderer,
  RGBA,
  TextAttributes,
} from "@opentui/core"

import type { Theme } from "../theme.js"

export interface ConfirmPopupOptions {
  id?: string
  title?: string
  message: string
  confirmLabel?: string
  cancelLabel?: string
  theme: Theme
  onConfirm: () => void
  onCancel: () => void
}

export interface ConfirmPopupInstance {
  root: BoxRenderable
  destroy: () => void
}

export function createConfirmPopup(
  renderer: CliRenderer,
  opts: ConfirmPopupOptions,
): ConfirmPopupInstance {
  const {
    id = "confirm-popup",
    title = "Confirm",
    message,
    confirmLabel = "Yes (y)",
    cancelLabel = "No (n/Esc)",
    theme,
    onConfirm,
    onCancel,
  } = opts

  // ── Overlay backdrop ──────────────────────────────────────────────────
  const overlay = new BoxRenderable(renderer, {
    id: `${id}-overlay`,
    position: "absolute" as any,
    width: "100%" as any,
    height: "100%" as any,
    justifyContent: "center",
    alignItems: "center",
    backgroundColor: RGBA.fromInts(0, 0, 0, 100),
  })

  // ── Dialog box ────────────────────────────────────────────────────────
  const dialog = new BoxRenderable(renderer, {
    id: `${id}-dialog`,
    width: 50,
    backgroundColor: RGBA.fromHex(theme.colors.bg),
    padding: 3,
    flexDirection: "column",
    gap: 2,
    title,
    titleAlignment: "center" as any,
  })

  const msgText = new TextRenderable(renderer, {
    id: `${id}-msg`,
    content: message,
    fg: theme.colors.textPrimary,
  })

  const buttonRow = new BoxRenderable(renderer, {
    id: `${id}-buttons`,
    flexDirection: "row",
    justifyContent: "center",
    gap: 4,
    height: 1,
    marginTop: 2,
  })

  const confirmBtn = new TextRenderable(renderer, {
    id: `${id}-confirm`,
    content: `  ${confirmLabel}  `,
    fg: theme.colors.textInverse,
    bg: theme.colors.accent,
    attributes: TextAttributes.BOLD,
  })

  const cancelBtn = new TextRenderable(renderer, {
    id: `${id}-cancel`,
    content: `  ${cancelLabel}  `,
    fg: theme.colors.textPrimary,
    bg: theme.colors.surface,
  })

  buttonRow.add(confirmBtn)
  buttonRow.add(cancelBtn)

  dialog.add(msgText)
  dialog.add(buttonRow)
  overlay.add(dialog)

  // ── Keyboard handling via global input handler ────────────────────────
  const handler = (sequence: string) => {
    if (sequence === "y" || sequence === "Y") {
      destroy()
      onConfirm()
      return true
    }
    if (sequence === "n" || sequence === "N" || sequence === "\x1b") {
      destroy()
      onCancel()
      return true
    }
    return true // consume all input while popup is open
  }
  renderer.prependInputHandler(handler)

  // Mouse handlers
  confirmBtn.onMouseDown = () => {
    destroy()
    onConfirm()
  }
  cancelBtn.onMouseDown = () => {
    destroy()
    onCancel()
  }

  function destroy() {
    renderer.removeInputHandler?.(handler)
    try {
      renderer.root.remove(`${id}-overlay`)
    } catch {
      // Already removed
    }
  }

  return { root: overlay, destroy }
}
