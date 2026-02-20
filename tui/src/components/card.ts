/**
 * Card — borderless flat card for history entries, OneContext style.
 *
 * Layout:
 *   ██ Title Text (bold)                   Share   +   ↩   x
 *   Description/subtitle in gray
 */

import {
  BoxRenderable,
  TextRenderable,
  type CliRenderer,
  RGBA,
  TextAttributes,
} from "@opentui/core"

import type { Theme } from "../theme.js"

export interface CardAction {
  label: string
  handler: () => void
}

export interface CardOptions {
  id: string
  text: string
  subtitle: string
  actions: CardAction[]
  selected?: boolean
  width?: number | string
  theme: Theme
}

export interface CardInstance {
  root: BoxRenderable
  setSelected: (v: boolean) => void
  setText: (text: string) => void
  setSubtitle: (sub: string) => void
  updateTheme: (theme: Theme) => void
}

export function createCard(
  renderer: CliRenderer,
  opts: CardOptions,
): CardInstance {
  const {
    id,
    text,
    subtitle,
    actions,
    selected = false,
    width = "100%",
  } = opts

  let theme = opts.theme

  const root = new BoxRenderable(renderer, {
    id: `${id}-card`,
    width: width as any,
    flexDirection: "column",
    paddingLeft: 2,
    paddingRight: 2,
    paddingTop: 1,
    paddingBottom: 1,
    marginBottom: 1,
    backgroundColor: RGBA.fromHex(selected ? theme.colors.selected : theme.colors.surface),
  })

  // ── Title row: indicator + text ... actions ────────────────────────
  const titleRow = new BoxRenderable(renderer, {
    id: `${id}-title-row`,
    width: "100%" as any,
    flexDirection: "row",
    justifyContent: "space-between",
    height: 1,
  })

  const titleLeft = new BoxRenderable(renderer, {
    id: `${id}-title-left`,
    flexDirection: "row",
    gap: 1,
    height: 1,
  })

  const indicator = new TextRenderable(renderer, {
    id: `${id}-ind`,
    content: "██",
    fg: selected ? theme.colors.indicator : theme.colors.inactive,
  })

  const textEl = new TextRenderable(renderer, {
    id: `${id}-text`,
    content: text,
    fg: theme.colors.textPrimary,
    attributes: TextAttributes.BOLD,
  })

  titleLeft.add(indicator)
  titleLeft.add(textEl)

  // Right side: action labels
  const actionRow = new BoxRenderable(renderer, {
    id: `${id}-actions`,
    flexDirection: "row",
    gap: 3,
    height: 1,
  })

  const actionSymbols = ["Share", "+", "↩", "x"]
  for (let i = 0; i < actions.length; i++) {
    const action = actions[i]
    const sym = actionSymbols[i] || action.label
    const btn = new TextRenderable(renderer, {
      id: `${id}-btn-${i}`,
      content: sym,
      fg: theme.colors.textSecondary,
    })
    
    btn.onMouseDown = action.handler
    
    actionRow.add(btn)
  }

  titleRow.add(titleLeft)
  titleRow.add(actionRow)

  // ── Subtitle ──────────────────────────────────────────────────────────
  const subtitleEl = new TextRenderable(renderer, {
    id: `${id}-subtitle`,
    content: subtitle,
    fg: theme.colors.textSecondary,
  })

  root.add(titleRow)
  root.add(subtitleEl)

  function setSelected(v: boolean) {
    root.backgroundColor = RGBA.fromHex(v ? theme.colors.selected : theme.colors.surface)
    indicator.fg = RGBA.fromHex(v ? theme.colors.indicator : theme.colors.inactive)
  }

  function setText(t: string) {
    textEl.content = t
  }

  function setSubtitle(s: string) {
    subtitleEl.content = s
  }

  function updateTheme(newTheme: Theme) {
    theme = newTheme
    root.backgroundColor = RGBA.fromHex(theme.colors.surface)
    textEl.fg = RGBA.fromHex(theme.colors.textPrimary)
    subtitleEl.fg = RGBA.fromHex(theme.colors.textSecondary)
    indicator.fg = RGBA.fromHex(theme.colors.inactive)
    
    // Update action buttons
    for (let i = 0; i < actionRow.children.length; i++) {
      const btn = actionRow.children[i] as TextRenderable
      btn.fg = RGBA.fromHex(theme.colors.textSecondary)
    }
  }

  return { root, setSelected, setText, setSubtitle, updateTheme }
}
