/**
 * Toggle — OneContext-style slide toggle.
 *
 *   Label                            Off  ██  On
 */

import {
  BoxRenderable,
  TextRenderable,
  type CliRenderer,
  RGBA,
} from "@opentui/core"

import type { Theme } from "../theme.js"

export interface ToggleOptions {
  id: string
  label: string
  value: boolean
  theme: Theme
  onChange?: (newValue: boolean) => void
}

export interface ToggleInstance {
  root: BoxRenderable
  value: boolean
  setValue: (v: boolean) => void
  toggle: () => void
  focus: () => void
  blur: () => void
  updateTheme: (theme: Theme) => void
}

export function createToggle(
  renderer: CliRenderer,
  opts: ToggleOptions,
): ToggleInstance {
  const { id, label, value: initValue, onChange } = opts

  let value = initValue
  let theme = opts.theme

  const root = new BoxRenderable(renderer, {
    id: `${id}-toggle`,
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
    fg: theme.colors.textPrimary,
  })

  const controlRow = new BoxRenderable(renderer, {
    id: `${id}-ctrl`,
    flexDirection: "row",
    gap: 1,
    height: 1,
  })

  const offLabel = new TextRenderable(renderer, {
    id: `${id}-off`,
    content: "Off",
    fg: value ? theme.colors.textMuted : theme.colors.textPrimary,
  })

  const switchBlock = new TextRenderable(renderer, {
    id: `${id}-switch`,
    content: " ██ ",
    fg: value ? theme.colors.active : theme.colors.inactive,
  })

  const onLabel = new TextRenderable(renderer, {
    id: `${id}-on`,
    content: "On",
    fg: value ? theme.colors.textPrimary : theme.colors.textMuted,
  })

  controlRow.add(offLabel)
  controlRow.add(switchBlock)
  controlRow.add(onLabel)

  root.add(labelText)
  root.add(controlRow)

  // Mouse interactions
  root.onMouseDown = () => {
    toggleValue()
  }

  function updateDisplay() {
    offLabel.fg = RGBA.fromHex(value ? theme.colors.textMuted : theme.colors.textPrimary)
    onLabel.fg = RGBA.fromHex(value ? theme.colors.textPrimary : theme.colors.textMuted)
    switchBlock.fg = RGBA.fromHex(value ? theme.colors.active : theme.colors.inactive)
  }

  function toggleValue() {
    value = !value
    updateDisplay()
    onChange?.(value)
  }

  function setValue(v: boolean) {
    value = v
    updateDisplay()
  }

  function focusToggle() {
    labelText.content = `▶ ${label}`
    labelText.fg = RGBA.fromHex(theme.colors.indicator)
    root.backgroundColor = RGBA.fromHex(theme.colors.selected)
  }

  function blurToggle() {
    labelText.content = label
    labelText.fg = RGBA.fromHex(theme.colors.textPrimary)
    root.backgroundColor = RGBA.fromHex("transparent")
  }

  function updateTheme(newTheme: Theme) {
    theme = newTheme
    labelText.fg = RGBA.fromHex(theme.colors.textPrimary)
    root.backgroundColor = RGBA.fromHex("transparent")
    updateDisplay()
  }

  return { root, get value() { return value }, setValue, toggle: toggleValue, focus: focusToggle, blur: blurToggle, updateTheme }
}
