import {
  BoxRenderable,
  TextRenderable,
  SelectRenderable,
  SelectRenderableEvents,
  type CliRenderer,
  RGBA,
} from "@opentui/core"

import type { Theme } from "../theme.js"

const POPUP_WIDTH = 96
const POPUP_HEIGHT = 16
const POPUP_LIST_HEIGHT = 11

export interface SelectFieldOption {
  name: string
  description?: string
}

export interface SelectFieldOptions {
  id: string
  label: string
  options: SelectFieldOption[]
  selectedIndex?: number
  theme: Theme
  onChange?: (index: number, option: SelectFieldOption) => void
}

export interface SelectFieldInstance {
  root: BoxRenderable
  focus: () => void
  blur: () => void
  open: () => void
  close: () => void
  destroy: () => void
  setOptions: (options: SelectFieldOption[], selectedIndex?: number) => void
  setSelectedIndex: (index: number) => void
  isOpen: () => boolean
}

export function createSelectField(
  renderer: CliRenderer,
  opts: SelectFieldOptions,
): SelectFieldInstance {
  const { id, label, onChange } = opts
  let theme = opts.theme
  let options = opts.options.slice()
  let selectedIndex = Math.max(0, Math.min(opts.selectedIndex ?? 0, Math.max(0, options.length - 1)))
  let draftIndex = selectedIndex
  let open = false
  let isFocused = false
  let programmaticPopupSelection = false

  let overlay: BoxRenderable | null = null
  let popupSelect: SelectRenderable | null = null
  let popupInputHandler: ((seq: string) => boolean) | null = null

  const root = new BoxRenderable(renderer, {
    id: `${id}-field`,
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

  const valueText = new TextRenderable(renderer, {
    id: `${id}-value`,
    content: options[selectedIndex]?.name ? `${options[selectedIndex].name}  ▾` : "No options",
    fg: theme.colors.textPrimary,
  })

  root.add(labelText)
  root.add(valueText)

  function renderRow() {
    labelText.content = isFocused ? `▶ ${label}` : label
    labelText.fg = RGBA.fromHex(isFocused ? theme.colors.indicator : theme.colors.textSecondary)
    valueText.content = options[selectedIndex]?.name ? `${options[selectedIndex].name}  ▾` : "No options"
    valueText.fg = RGBA.fromHex(options.length > 0 ? theme.colors.textPrimary : theme.colors.textMuted)
    root.backgroundColor = RGBA.fromHex(isFocused ? theme.colors.selected : theme.colors.surface)
  }

  function commitSelectedIndex(index: number) {
    if (options.length === 0) return
    const next = Math.max(0, Math.min(index, options.length - 1))
    selectedIndex = next
    renderRow()
    onChange?.(selectedIndex, options[selectedIndex])
  }

  function closePopup() {
    if (!open) return
    open = false
    if (popupInputHandler) {
      renderer.removeInputHandler?.(popupInputHandler)
      popupInputHandler = null
    }
    if (overlay) {
      try { renderer.root.remove(overlay.id) } catch {}
    }
    overlay = null
    popupSelect = null
  }

  function openPopup() {
    if (open || options.length === 0) return
    open = true
    draftIndex = selectedIndex

    overlay = new BoxRenderable(renderer, {
      id: `${id}-overlay`,
      position: "absolute" as any,
      width: "100%" as any,
      height: "100%" as any,
      justifyContent: "center",
      alignItems: "center",
      backgroundColor: RGBA.fromInts(0, 0, 0, 90),
    })
    overlay.onMouseDown = () => {
      closePopup()
    }

    const dialog = new BoxRenderable(renderer, {
      id: `${id}-dialog`,
      width: POPUP_WIDTH as any,
      height: POPUP_HEIGHT as any,
      backgroundColor: RGBA.fromHex(theme.colors.bg),
      flexDirection: "column",
      padding: 2,
      gap: 1,
      title: label,
      titleAlignment: "left" as any,
    })

    popupSelect = new SelectRenderable(renderer, {
      id: `${id}-popup-select`,
      width: "100%" as any,
      height: POPUP_LIST_HEIGHT,
      options: options.map((o) => ({
        name: o.name,
        // Keep every row at the same height across all selectors.
        description: o.description && o.description.trim().length > 0 ? o.description : " ",
      })),
      showDescription: true,
      backgroundColor: RGBA.fromHex(theme.colors.surface),
      focusedBackgroundColor: RGBA.fromHex(theme.colors.hover),
      focusedTextColor: RGBA.fromHex(theme.colors.textPrimary),
      selectedBackgroundColor: RGBA.fromHex(theme.colors.selected),
      selectedTextColor: RGBA.fromHex(theme.colors.textPrimary),
      textColor: RGBA.fromHex(theme.colors.textSecondary),
      descriptionColor: RGBA.fromHex(theme.colors.textMuted),
    })

    popupSelect.setSelectedIndex(draftIndex)
    popupSelect.on(SelectRenderableEvents.SELECTION_CHANGED, (index: number) => {
      draftIndex = index
      if (!programmaticPopupSelection) {
        // Mouse selection: commit immediately.
        commitSelectedIndex(draftIndex)
        closePopup()
      }
    })

    const hint = new TextRenderable(renderer, {
      id: `${id}-hint`,
      content: "↑/↓ or j/k to navigate • Enter/Esc to close",
      fg: theme.colors.textMuted,
    })

    dialog.add(popupSelect)
    dialog.add(hint)
    overlay.add(dialog)
    dialog.onMouseDown = () => {
      // prevent backdrop click-to-close when interacting inside dialog
    }
    renderer.root.add(overlay)
    popupSelect.focus()

    popupInputHandler = (seq: string) => {
      if (!open) return false
      if (seq === "\x03") return false // let Ctrl+C propagate to global exit handling
      if (seq === "\x1b") { closePopup(); return true }
      if (seq === "\r" || seq === "\n") { commitSelectedIndex(draftIndex); closePopup(); return true }
      if (seq === "\x1b[B" || seq === "j") {
        draftIndex = Math.min(draftIndex + 1, options.length - 1)
        if (popupSelect) {
          programmaticPopupSelection = true
          popupSelect.setSelectedIndex(draftIndex)
          programmaticPopupSelection = false
        }
        return true
      }
      if (seq === "\x1b[A" || seq === "k") {
        draftIndex = Math.max(draftIndex - 1, 0)
        if (popupSelect) {
          programmaticPopupSelection = true
          popupSelect.setSelectedIndex(draftIndex)
          programmaticPopupSelection = false
        }
        return true
      }
      return true
    }
    renderer.prependInputHandler(popupInputHandler)
  }

  root.onMouseDown = () => {
    openPopup()
  }

  function focus() {
    isFocused = true
    renderRow()
  }

  function blur() {
    isFocused = false
    renderRow()
  }

  function setOptions(nextOptions: SelectFieldOption[], nextSelectedIndex = 0) {
    options = nextOptions.slice()
    selectedIndex = Math.max(0, Math.min(nextSelectedIndex, Math.max(0, options.length - 1)))
    draftIndex = selectedIndex
    renderRow()
    if (open) {
      closePopup()
    }
  }

  function destroy() {
    closePopup()
  }

  renderRow()

  return {
    root,
    focus,
    blur,
    open: openPopup,
    close: closePopup,
    destroy,
    setOptions,
    setSelectedIndex: commitSelectedIndex,
    isOpen: () => open,
  }
}
