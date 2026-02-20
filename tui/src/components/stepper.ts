/**
 * Stepper — cycles through a predefined list of values with ◀ / ▶ arrows.
 *
 *   Label                      ◀  16000  ▶
 *
 * Focused:
 *   ▶ Label                    ◀  16000  ▶
 *
 * Keyboard: Left/Right arrows cycle through values (dispatched externally
 * by the config panel's prependInputHandler, same pattern as Toggle + Space).
 */

import {
  BoxRenderable,
  TextRenderable,
  type CliRenderer,
  RGBA,
} from "@opentui/core"

import type { Theme } from "../theme.js"

export interface StepperOptions {
  id: string
  label: string
  values: string[]
  selectedIndex?: number
  theme: Theme
  onChange?: (value: string, index: number) => void
}

export interface StepperInstance {
  root: BoxRenderable
  readonly value: string
  readonly index: number
  next: () => void
  prev: () => void
  setIndex: (i: number) => void
  focus: () => void
  blur: () => void
  updateTheme: (theme: Theme) => void
}

export function createStepper(
  renderer: CliRenderer,
  opts: StepperOptions,
): StepperInstance {
  const { id, label, values, onChange } = opts

  let idx = Math.max(0, Math.min(opts.selectedIndex ?? 0, values.length - 1))
  let theme = opts.theme

  // ── Layout ────────────────────────────────────────────────────────
  const root = new BoxRenderable(renderer, {
    id: `${id}-stepper`,
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

  const controlRow = new BoxRenderable(renderer, {
    id: `${id}-ctrl`,
    flexDirection: "row",
    gap: 1,
    height: 1,
  })

  const leftArrow = new TextRenderable(renderer, {
    id: `${id}-left`,
    content: "◀",
    fg: theme.colors.textMuted,
  })

  const valueText = new TextRenderable(renderer, {
    id: `${id}-value`,
    content: values[idx] ?? "",
    fg: theme.colors.textPrimary,
  })

  const rightArrow = new TextRenderable(renderer, {
    id: `${id}-right`,
    content: "▶",
    fg: theme.colors.textMuted,
  })

  controlRow.add(leftArrow)
  controlRow.add(valueText)
  controlRow.add(rightArrow)

  root.add(labelText)
  root.add(controlRow)

  // Clickable arrows
  leftArrow.onMouseDown = () => {
    prev()
  }

  rightArrow.onMouseDown = () => {
    next()
  }

  // ── State mutations ───────────────────────────────────────────────
  function updateDisplay() {
    valueText.content = values[idx] ?? ""
  }

  function next() {
    idx = (idx + 1) % values.length
    updateDisplay()
    onChange?.(values[idx], idx)
  }

  function prev() {
    idx = (idx - 1 + values.length) % values.length
    updateDisplay()
    onChange?.(values[idx], idx)
  }

  function setIndex(i: number) {
    idx = Math.max(0, Math.min(i, values.length - 1))
    updateDisplay()
  }

  function focus() {
    labelText.content = `▶ ${label}`
    labelText.fg = RGBA.fromHex(theme.colors.indicator)
    leftArrow.fg = RGBA.fromHex(theme.colors.textPrimary)
    rightArrow.fg = RGBA.fromHex(theme.colors.textPrimary)
    root.backgroundColor = RGBA.fromHex(theme.colors.selected)
  }

  function blur() {
    labelText.content = label
    labelText.fg = RGBA.fromHex(theme.colors.textSecondary)
    leftArrow.fg = RGBA.fromHex(theme.colors.textMuted)
    rightArrow.fg = RGBA.fromHex(theme.colors.textMuted)
    root.backgroundColor = RGBA.fromHex(theme.colors.surface)
  }

  function updateTheme(newTheme: Theme) {
    theme = newTheme
    labelText.fg = RGBA.fromHex(theme.colors.textSecondary)
    valueText.fg = RGBA.fromHex(theme.colors.textPrimary)
    leftArrow.fg = RGBA.fromHex(theme.colors.textMuted)
    rightArrow.fg = RGBA.fromHex(theme.colors.textMuted)
    root.backgroundColor = RGBA.fromHex(theme.colors.surface)
  }

  return {
    root,
    get value() { return values[idx] ?? "" },
    get index() { return idx },
    next,
    prev,
    setIndex,
    focus,
    blur,
    updateTheme,
  }
}
