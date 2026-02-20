/**
 * KeyCapture — captures key combinations like "Ctrl+Shift+Space".
 *
 * When focused, shows "⌨ Press key combo..." and records the next key press.
 * When blurred, displays the current combo string.
 *
 * Tab / Shift+Tab are NOT captured — they navigate to the next/prev widget.
 */

import {
  BoxRenderable,
  TextRenderable,
  type CliRenderer,
  RGBA,
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
    backgroundColor: RGBA.fromHex("transparent"),
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
    if (isFocused) {
      labelText.content = `▶ ${label}`
      labelText.fg = RGBA.fromHex(theme.colors.indicator)
      comboText.content = "⌨ Press key combo..."
      comboText.fg = RGBA.fromHex(theme.colors.textPrimary)
      root.backgroundColor = RGBA.fromHex(theme.colors.selected)
    } else {
      labelText.content = label
      labelText.fg = RGBA.fromHex(theme.colors.textSecondary)
      comboText.content = value || "(not set)"
      comboText.fg = RGBA.fromHex(value ? theme.colors.textPrimary : theme.colors.textMuted)
      root.backgroundColor = RGBA.fromHex("transparent")
    }
  }

  function formatKeyCombo(key: any): string {
    const parts: string[] = []
    if (key.ctrl)  parts.push("Ctrl")
    if (key.meta || key.super) {
      parts.push(process.platform === "darwin" ? "Cmd" : "Win")
    }
    if (key.shift) parts.push("Shift")
    if (key.alt)   parts.push("Alt")

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

    const rawName = key.name || ""
    // Bare modifier-only presses — ignore
    if (["control", "shift", "alt", "meta", "super"].includes(rawName)) return ""

    const keyName = nameMap[rawName] || (rawName.length === 1
      ? rawName.toUpperCase()
      : rawName.charAt(0).toUpperCase() + rawName.slice(1))
    parts.push(keyName)

    return parts.join("+")
  }

  function focusCapture() {
    if (isFocused) return
    isFocused = true

    // Make sure no OpenTUI renderable is stealing the key events
    ;(renderer as any).focusRenderable(null)

    updateDisplay()

    keypressHandler = (key: any) => {
      if (!isFocused) return

      // Tab / Shift+Tab — let the config nav handler deal with it
      if (key.name === "tab") return

      // Escape cancels without changing value
      if (key.name === "escape" && !key.ctrl && !key.shift && !key.meta) {
        blurCapture()
        return
      }

      const combo = formatKeyCombo(key)
      if (!combo) return // bare modifier — wait for a real key

      value = combo
      isFocused = false
      updateDisplay()
      // Remove listener before calling onChange so a refocus doesn't double-register
      if (keypressHandler) {
        renderer.keyInput.removeListener("keypress", keypressHandler)
        keypressHandler = null
      }
      onChange?.(combo)
    }

    renderer.keyInput.on("keypress", keypressHandler)
  }

  function blurCapture() {
    if (!isFocused) return
    isFocused = false
    updateDisplay()
    if (keypressHandler) {
      renderer.keyInput.removeListener("keypress", keypressHandler)
      keypressHandler = null
    }
  }

  function setValue(v: string) {
    value = v
    if (!isFocused) updateDisplay()
  }

  function destroy() {
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
    blur: blurCapture,
    destroy,
    updateTheme,
  }
}
