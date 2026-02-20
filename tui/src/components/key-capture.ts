/**
 * KeyCapture — captures key combinations like "Ctrl+Shift+Space".
 *
 * When focused, shows "⌨ Press key combo..." and records the next key press.
 * When blurred, displays the current combo string.
 *
 * Plain Esc is not captured — it closes the popup.
 */

import {
  BoxRenderable,
  TextRenderable,
  type CliRenderer,
  RGBA,
  TextAttributes,
} from "@opentui/core"

import type { Theme } from "../theme.js"

export interface KeyCaptureOptions {
  id: string
  label: string
  value: string
  theme: Theme
  onChange?: (combo: string) => void
}

export interface KeyCaptureInstance {
  root: BoxRenderable
  readonly value: string
  setValue: (v: string) => void
  focus: () => void
  open: () => void
  cancelCapture: () => void
  isCapturing: () => boolean
  blur: () => void
  destroy: () => void
  updateTheme: (theme: Theme) => void
}

export function createKeyCapture(
  renderer: CliRenderer,
  opts: KeyCaptureOptions,
): KeyCaptureInstance {
  const { id, label, onChange } = opts

  let value = opts.value
  let isFocused = false
  let isCapturingCombo = false
  let overlay: BoxRenderable | null = null
  let keypressHandler: ((key: any) => void) | null = null
  let theme = opts.theme

  const root = new BoxRenderable(renderer, {
    id: `${id}-keycap`,
    width: "100%" as any,
    flexDirection: "row",
    justifyContent: "space-between",
    height: 1,
    paddingLeft: 1,
    paddingRight: 1,
    backgroundColor: RGBA.fromHex(theme.colors.surface),
  })

  const labelText = new TextRenderable(renderer, {
    id: `${id}-label`,
    content: label,
    fg: theme.colors.textSecondary,
  })

  const comboText = new TextRenderable(renderer, {
    id: `${id}-combo`,
    content: value || "(not set)",
    fg: value ? theme.colors.textPrimary : theme.colors.textMuted,
  })

  root.add(labelText)
  root.add(comboText)

  root.onMouseDown = () => {
    focusCapture()
  }

  function updateDisplay() {
    if (isCapturingCombo) {
      labelText.content = `▶ ${label}`
      labelText.fg = RGBA.fromHex(theme.colors.indicator)
      comboText.content = "⌨ Recording..."
      comboText.fg = RGBA.fromHex(theme.colors.textPrimary)
      root.backgroundColor = RGBA.fromHex(theme.colors.selected)
      return
    }

    if (isFocused) {
      labelText.content = `▶ ${label}`
      labelText.fg = RGBA.fromHex(theme.colors.indicator)
      comboText.content = "Press Enter to record"
      comboText.fg = RGBA.fromHex(theme.colors.textMuted)
      root.backgroundColor = RGBA.fromHex(theme.colors.selected)
    } else {
      labelText.content = label
      labelText.fg = RGBA.fromHex(theme.colors.textSecondary)
      comboText.content = value || "(not set)"
      comboText.fg = RGBA.fromHex(value ? theme.colors.textPrimary : theme.colors.textMuted)
      root.backgroundColor = RGBA.fromHex(theme.colors.surface)
    }
  }

  function normalizePrintableKey(rawName: string): string {
    const shiftedSymbolMap: Record<string, string> = {
      ")": "0",
      "!": "1",
      "@": "2",
      "#": "3",
      "$": "4",
      "%": "5",
      "^": "6",
      "&": "7",
      "*": "8",
      "(": "9",
      "_": "-",
      "+": "=",
      "{": "[",
      "}": "]",
      "|": "\\",
      ":": ";",
      "\"": "'",
      "<": ",",
      ">": ".",
      "?": "/",
      "~": "`",
    }

    if (shiftedSymbolMap[rawName]) return shiftedSymbolMap[rawName]
    return rawName
  }

  function resolveRawKeyName(key: any): string {
    if (typeof key.name === "string" && key.name.length > 0) return key.name.toLowerCase()

    const seq = typeof key.sequence === "string" ? key.sequence : ""
    if (seq.length !== 1) return ""

    const code = seq.charCodeAt(0)
    const isPrintableAscii = code >= 32 && code <= 126
    if (isPrintableAscii) return seq

    // Common terminal control-sequence decoding for Ctrl+key combinations.
    if (key.ctrl) {
      if (code >= 1 && code <= 26) return String.fromCharCode(code + 96) // Ctrl+A..Ctrl+Z
      const ctrlCodeMap: Record<number, string> = {
        0: "2",  // often Ctrl+2 / Ctrl+Space
        28: "\\", // Ctrl+\
        29: "]",  // Ctrl+]
        30: "6",  // Ctrl+6
        31: "-",  // Ctrl+-
      }
      if (ctrlCodeMap[code]) return ctrlCodeMap[code]
    }

    return ""
  }

  function formatKeyCombo(key: any): string {
    const modifiers: string[] = []
    if (key.ctrl) modifiers.push("Ctrl")
    if (key.meta || key.super) modifiers.push(process.platform === "darwin" ? "Cmd" : "Win")
    if (key.shift) modifiers.push("Shift")
    if (key.alt) modifiers.push("Alt")

    // Allow up to two modifiers plus one main key (e.g., Cmd+Shift+0).
    if (modifiers.length > 2) return ""

    // Map key names to readable strings
    const nameMap: Record<string, string> = {
      space: "Space",
      return: "Enter",
      escape: "Escape",
      tab: "Tab",
      backspace: "Backspace",
      delete: "Delete",
      up: "Up",
      down: "Down",
      left: "Left",
      right: "Right",
      home: "Home",
      end: "End",
      pageup: "PageUp",
      pagedown: "PageDown",
      f1: "F1",  f2: "F2",  f3: "F3",  f4: "F4",
      f5: "F5",  f6: "F6",  f7: "F7",  f8: "F8",
      f9: "F9",  f10: "F10", f11: "F11", f12: "F12",
    }

    const rawName = resolveRawKeyName(key)
    // Bare modifier-only presses — ignore
    if (["control", "shift", "alt", "meta", "super"].includes(rawName)) return ""

    const normalizedName = normalizePrintableKey(rawName)
    if (!normalizedName) return ""
    const keyName = nameMap[normalizedName] || (normalizedName.length === 1
      ? normalizedName.toUpperCase()
      : rawName.charAt(0).toUpperCase() + rawName.slice(1))
    const parts = [...modifiers, keyName]

    return parts.join("+")
  }

  function destroyPopup() {
    if (!overlay) return
    try {
      renderer.root.remove(`${id}-keycap-overlay`)
    } catch {}
    overlay = null
  }

  function stopCapture() {
    isCapturingCombo = false
    if (keypressHandler) {
      renderer.keyInput.removeListener("keypress", keypressHandler)
      keypressHandler = null
    }
    destroyPopup()
    updateDisplay()
  }

  function openCapture() {
    if (!isFocused || isCapturingCombo) return
    isCapturingCombo = true
    ;(renderer as any).focusRenderable(null)

    overlay = new BoxRenderable(renderer, {
      id: `${id}-keycap-overlay`,
      position: "absolute" as any,
      width: "100%" as any,
      height: "100%" as any,
      justifyContent: "center",
      alignItems: "center",
      backgroundColor: RGBA.fromInts(0, 0, 0, 90),
    })

    const popup = new BoxRenderable(renderer, {
      id: `${id}-keycap-popup`,
      width: 56,
      flexDirection: "column",
      gap: 1,
      padding: 2,
      backgroundColor: RGBA.fromHex(theme.colors.bg),
      title: "Record Keybinding",
      titleAlignment: "center" as any,
    })

    const titleText = new TextRenderable(renderer, {
      id: `${id}-keycap-title`,
      content: label,
      fg: RGBA.fromHex(theme.colors.textPrimary),
      attributes: TextAttributes.BOLD,
    })

    const helpText = new TextRenderable(renderer, {
      id: `${id}-keycap-help`,
      content: "Press key combo (max 2 modifiers + 1 key, Esc to cancel)",
      fg: RGBA.fromHex(theme.colors.textSecondary),
    })

    popup.add(titleText)
    popup.add(helpText)
    overlay.add(popup)
    renderer.root.add(overlay)
    updateDisplay()

    keypressHandler = (key: any) => {
      if (!isCapturingCombo) return

      // Escape cancels capture (only plain Esc)
      if (key.name === "escape" && !key.ctrl && !key.shift && !key.meta && !key.alt) {
        stopCapture()
        return
      }

      const combo = formatKeyCombo(key)
      if (!combo) return

      value = combo
      stopCapture()
      onChange?.(combo)
    }
    renderer.keyInput.on("keypress", keypressHandler)
  }

  function focusCapture() {
    if (isFocused) return
    isFocused = true
    updateDisplay()
  }

  function blurCapture() {
    if (!isFocused) return
    stopCapture()
    isFocused = false
    updateDisplay()
  }

  function setValue(v: string) {
    value = v
    if (!isFocused) updateDisplay()
  }

  function destroy() {
    stopCapture()
    blurCapture()
  }

  function updateTheme(newTheme: Theme) {
    theme = newTheme
    updateDisplay()
  }

  return {
    root,
    get value() { return value },
    setValue,
    focus: focusCapture,
    open: openCapture,
    cancelCapture: stopCapture,
    isCapturing: () => isCapturingCombo,
    blur: blurCapture,
    destroy,
    updateTheme,
  }
}
