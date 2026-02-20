/**
 * Config Panel — flat minimalist settings, always-visible sections.
 *
 * Sections:
 *   1. Model Selection
 *   2. Key Bindings
 *   3. Device Selection
 *   4. Audio Settings
 *   5. VAD Settings
 *   6. Post Processing
 *   7. Injection
 *   8. History
 */

import {
  BoxRenderable,
  TextRenderable,
  ScrollBoxRenderable,
  SelectRenderable,
  SelectRenderableEvents,
  type CliRenderer,
  RGBA,
  TextAttributes,
} from "@opentui/core"

import type { AppState } from "../app.js"
import { saveConfig, type VoxConfig } from "../data/config.js"
import { getModelRegistry, listDevicesWithError } from "../data/cli.js"
import { createToggle, type ToggleInstance } from "../components/toggle.js"
import { createKeyCapture, type KeyCaptureInstance } from "../components/key-capture.js"
import { createStepper, type StepperInstance } from "../components/stepper.js"

export interface ConfigPanelCallbacks {
  onDirty: () => void
  onSaved: () => void
  onStatusMessage: (msg: string) => void
  onEscape?: () => void
}

export interface ConfigPanelInstance {
  root: BoxRenderable
  save: () => void
  focusFirst: () => void
  blurAll: () => void
  hasFocus: () => boolean
  destroy: () => void
}

export function createConfigPanel(
  renderer: CliRenderer,
  state: AppState,
  callbacks: ConfigPanelCallbacks,
): ConfigPanelInstance {
  const config = state.config
  const theme = state.theme

  function markDirty() {
    state.configDirty = true
    callbacks.onDirty()
  }

  // ── Root ─────────────────────────────────────────────────────────────
  const root = new BoxRenderable(renderer, {
    id: "config-panel",
    width: "100%" as any,
    height: "100%" as any,
    flexDirection: "column",
  })

  // ── Top bar ──────────────────────────────────────────────────────────
  const topBar = new BoxRenderable(renderer, {
    id: "config-topbar",
    width: "100%" as any,
    height: 1,
    flexDirection: "row",
    justifyContent: "space-between",
    marginBottom: 2,
    paddingBottom: 1,
  })

  const titleText = new TextRenderable(renderer, {
    id: "config-title",
    content: "Configuration",
    fg: theme.colors.textPrimary,
    attributes: TextAttributes.BOLD,
  })

  const saveHint = new TextRenderable(renderer, {
    id: "config-save-hint",
    content: "Ctrl+S to save",
    fg: theme.colors.textSecondary,
  })

  topBar.add(titleText)
  topBar.add(saveHint)

  // ── Scrollable content ───────────────────────────────────────────────
  const scrollBox = new ScrollBoxRenderable(renderer, {
    id: "config-scroll",
    width: "100%" as any,
    height: "100%" as any,
    viewportCulling: false, // Keep all rendered for input focus
  })

  // Hide the scrollbar — we scroll programmatically based on focused widget
  scrollBox.verticalScrollBar.visible = false

  root.add(topBar)
  root.add(scrollBox)

  // ── Helper: create a flat section header + indented content box ─────
  function createSection(sectionId: string, title: string) {
    // Add a subtle divider before each section (except the first)
    const existingChildren = (scrollBox as any).content?.children
    if (existingChildren && existingChildren.length > 0) {
      const divider = new TextRenderable(renderer, {
        id: `${sectionId}-divider`,
        content: "",
        fg: theme.colors.border,
      })
      const dividerBox = new BoxRenderable(renderer, {
        id: `${sectionId}-divider-box`,
        width: "100%" as any,
        height: 1,
        marginBottom: 1,
        backgroundColor: RGBA.fromHex(theme.colors.border),
      })
      dividerBox.add(divider)
      scrollBox.add(dividerBox)
    }
    
    const header = new TextRenderable(renderer, {
      id: `${sectionId}-header`,
      content: `  ${title}`,
      fg: theme.colors.textPrimary,
      attributes: TextAttributes.BOLD,
    })
    const content = new BoxRenderable(renderer, {
      id: `${sectionId}-content`,
      width: "100%" as any,
      flexDirection: "column",
      paddingLeft: 4,
      paddingRight: 3,
      paddingTop: 1,
      paddingBottom: 1,
      marginBottom: 2,
      gap: 1,
    })
    scrollBox.add(header)
    scrollBox.add(content)
    return content
  }

  // ── 1. Model Selection ─────────────────────────────────────────────
  const modelContent = createSection("sec-model", "Model Selection")

  const models = getModelRegistry()
  const modelSelect = new SelectRenderable(renderer, {
    id: "model-select",
    width: "100%" as any,
    height: 6,
    options: models.map((m) => ({
      name: `${m.name}  (${m.size})`,
      description: `${m.speedFactor}x speed  •  ${m.memoryMb}MB RAM  •  ${m.description}`,
    })),
    backgroundColor: RGBA.fromHex(theme.colors.surface),
    focusedBackgroundColor: RGBA.fromHex(theme.colors.hover),
    focusedTextColor: RGBA.fromHex(theme.colors.textPrimary),
    selectedBackgroundColor: RGBA.fromHex(theme.colors.selected),
    selectedTextColor: RGBA.fromHex(theme.colors.textPrimary),
    textColor: RGBA.fromHex(theme.colors.textSecondary),
    descriptionColor: RGBA.fromHex(theme.colors.textMuted),
    showDescription: true,
  })

  // Pre-select current model
  const currentModelIdx = models.findIndex(
    (m) => config.model.model_path.includes(m.id.split("-").pop()!),
  )
  if (currentModelIdx >= 0) modelSelect.setSelectedIndex(currentModelIdx)

  modelSelect.on(SelectRenderableEvents.SELECTION_CHANGED, (index: number) => {
    config.model.model_path = models[index].id
    markDirty()
    callbacks.onStatusMessage(`Model → ${models[index].name}`)
    setTimeout(() => callbacks.onStatusMessage(""), 1500)
  })

  modelContent.add(modelSelect)

  // ── 2. Key Bindings ────────────────────────────────────────────────
  const hotkeyContent = createSection("sec-hotkey", "Key Bindings")

  // Push-to-talk trigger
  const triggerCapture = createKeyCapture(renderer, {
    id: "trigger-capture",
    label: "Push-to-talk trigger:",
    value: config.hotkey.trigger,
    theme,
    onChange: (combo) => { config.hotkey.trigger = combo; markDirty() },
  })

  // Toggle hotkey
  const toggleCapture = createKeyCapture(renderer, {
    id: "toggle-capture",
    label: "Toggle hotkey:",
    value: config.hotkey.toggle,
    theme,
    onChange: (combo) => { config.hotkey.toggle = combo; markDirty() },
  })

  // Mode selector
  const modeLabel = new TextRenderable(renderer, {
    id: "mode-label",
    content: "Mode:",
    fg: theme.colors.textSecondary,
  })
  const modeLabelBox = new BoxRenderable(renderer, {
    id: "mode-label-box",
    width: "100%" as any,
    marginTop: 1,
  })
  modeLabelBox.add(modeLabel)
  
  const modeSelect = new SelectRenderable(renderer, {
    id: "mode-select",
    width: 30,
    height: 3,
    options: [
      { name: "push-to-talk", description: "Hold key to dictate" },
      { name: "toggle", description: "Press to start/stop" },
    ],
    showDescription: true,
    backgroundColor: RGBA.fromHex(theme.colors.surface),
    focusedBackgroundColor: RGBA.fromHex(theme.colors.hover),
    focusedTextColor: RGBA.fromHex(theme.colors.textPrimary),
    selectedBackgroundColor: RGBA.fromHex(theme.colors.selected),
    textColor: RGBA.fromHex(theme.colors.textSecondary),
  })
  const modeIdx = config.hotkey.mode === "toggle" ? 1 : 0
  modeSelect.setSelectedIndex(modeIdx)
  modeSelect.on(SelectRenderableEvents.SELECTION_CHANGED, (index: number) => {
    config.hotkey.mode = index === 0 ? "push-to-talk" : "toggle"
    markDirty()
  })

  hotkeyContent.add(triggerCapture.root)
  hotkeyContent.add(toggleCapture.root)
  hotkeyContent.add(modeLabelBox)
  hotkeyContent.add(modeSelect)

  // ── 3. Device Selection ────────────────────────────────────────
  const deviceContent = createSection("sec-device", "Device Selection")

  const deviceLoading = new TextRenderable(renderer, {
    id: "device-loading",
    content: "Loading devices...",
    fg: theme.colors.textMuted,
  })
  deviceContent.add(deviceLoading)

  // Async load devices
  listDevicesWithError().then(({ devices, error }) => {
    try { deviceContent.remove("device-loading") } catch {}

    if (error) {
      const errText = new TextRenderable(renderer, {
        id: "device-error",
        content: `⚠ ${error}`,
        fg: theme.colors.textPrimary,
      })
      deviceContent.add(errText)
    }

    if (devices.length === 0) {
      const noDevices = new TextRenderable(renderer, {
        id: "no-devices",
        content: error ? "Could not list devices — check that `vox` is built" : "No audio input devices found",
        fg: theme.colors.textMuted,
      })
      deviceContent.add(noDevices)
      return
    }

    const deviceSelect = new SelectRenderable(renderer, {
      id: "device-select",
      width: "100%" as any,
      height: Math.min(devices.length + 1, 8),
      options: devices.map((d) => ({
        name: `${d.name}${d.isDefault ? " (default)" : ""}`,
        description: `${d.sampleRate}Hz, ${d.channels}ch`,
      })),
      backgroundColor: RGBA.fromHex(theme.colors.surface),
      focusedBackgroundColor: RGBA.fromHex(theme.colors.hover),
      focusedTextColor: RGBA.fromHex(theme.colors.textPrimary),
      selectedBackgroundColor: RGBA.fromHex(theme.colors.selected),
      textColor: RGBA.fromHex(theme.colors.textSecondary),
      showDescription: true,
    })

    // Pre-select current device
    const curIdx = devices.findIndex(
      (d) => d.name === config.audio.device || (config.audio.device === "default" && d.isDefault),
    )
    if (curIdx >= 0) deviceSelect.setSelectedIndex(curIdx)

    deviceSelect.on(SelectRenderableEvents.SELECTION_CHANGED, (index: number) => {
      config.audio.device = devices[index].name
      markDirty()
      callbacks.onStatusMessage(`Device → ${devices[index].name}`)
      setTimeout(() => callbacks.onStatusMessage(""), 1500)
    })

    deviceContent.add(deviceSelect)
    // Register in keyboard focus navigation (inserted before srStepper at index 4)
    focusables.splice(4, 0, { type: "select", widget: deviceSelect, scrollHint: 20 })
  })

  // ── 4. Audio Settings ──────────────────────────────────────────
  const audioContent = createSection("sec-audio", "Audio Settings")

  const SR_VALUES = ["8000", "11025", "16000", "22050", "44100", "48000", "96000"]
  const srStepper = createStepper(renderer, {
    id: "sr",
    label: "Sample Rate (Hz):",
    values: SR_VALUES,
    selectedIndex: SR_VALUES.indexOf(config.audio.sample_rate.toString()),
    theme,
    onChange: (v) => { config.audio.sample_rate = parseInt(v, 10); markDirty() },
  })

  const CHUNK_VALUES = ["50", "100", "150", "200", "300", "400", "500", "1000"]
  const chunkStepper = createStepper(renderer, {
    id: "chunk",
    label: "Chunk Duration (ms):",
    values: CHUNK_VALUES,
    selectedIndex: Math.max(0, CHUNK_VALUES.indexOf(config.audio.chunk_duration_ms.toString())),
    theme,
    onChange: (v) => { config.audio.chunk_duration_ms = parseInt(v, 10); markDirty() },
  })

  audioContent.add(srStepper.root)
  audioContent.add(chunkStepper.root)

  // ── 5. VAD Settings ────────────────────────────────────────────
  const vadContent = createSection("sec-vad", "VAD (Voice Activity Detection)")

  const vadEnabled = createToggle(renderer, {
    id: "vad-enabled",
    label: "Enabled",
    value: config.vad.enabled,
    theme,
    onChange: (v) => { config.vad.enabled = v; markDirty() },
  })

  const vadBackendLabel = new TextRenderable(renderer, {
    id: "vad-backend-label",
    content: "Backend:",
    fg: theme.colors.textSecondary,
  })
  const vadBackendLabelBox = new BoxRenderable(renderer, {
    id: "vad-backend-label-box",
    width: "100%" as any,
    marginTop: 1,
  })
  vadBackendLabelBox.add(vadBackendLabel)
  
  const vadBackendSelect = new SelectRenderable(renderer, {
    id: "vad-backend-select",
    width: 30,
    height: 4,
    options: [
      { name: "energy", description: "Simple energy-based detection" },
      { name: "silero", description: "Neural network-based (more accurate)" },
      { name: "webrtc", description: "WebRTC VAD library" },
    ],
    backgroundColor: RGBA.fromHex(theme.colors.surface),
    focusedBackgroundColor: RGBA.fromHex(theme.colors.hover),
    focusedTextColor: RGBA.fromHex(theme.colors.textPrimary),
    selectedBackgroundColor: RGBA.fromHex(theme.colors.selected),
    textColor: RGBA.fromHex(theme.colors.textSecondary),
    showDescription: true,
  })
  const vadIdx = ["energy", "silero", "webrtc"].indexOf(config.vad.backend)
  if (vadIdx >= 0) vadBackendSelect.setSelectedIndex(vadIdx)
  vadBackendSelect.on(SelectRenderableEvents.SELECTION_CHANGED, (index: number) => {
    config.vad.backend = ["energy", "silero", "webrtc"][index]
    markDirty()
  })

  // 0.00 – 1.00 in steps of 0.01  (101 values)
  const THRESH_VALUES = Array.from({ length: 101 }, (_, i) => (i / 100).toFixed(2))
  const vadThresholdStepper = createStepper(renderer, {
    id: "vad-threshold",
    label: "Threshold:",
    values: THRESH_VALUES,
    selectedIndex: Math.round(config.vad.threshold * 100),
    theme,
    onChange: (v) => { config.vad.threshold = parseFloat(v); markDirty() },
  })

  const vadAdaptive = createToggle(renderer, {
    id: "vad-adaptive",
    label: "Adaptive threshold",
    value: config.vad.adaptive,
    theme,
    onChange: (v) => { config.vad.adaptive = v; markDirty() },
  })

  vadContent.add(vadEnabled.root)
  vadContent.add(vadBackendLabelBox)
  vadContent.add(vadBackendSelect)
  vadContent.add(vadThresholdStepper.root)
  vadContent.add(vadAdaptive.root)

  // ── 6. Post Processing ─────────────────────────────────────────
  const ppContent = createSection("sec-pp", "Post Processing")

  const ppPunctuation = createToggle(renderer, {
    id: "pp-punct",
    label: "Auto-punctuation",
    value: config.post_processing.auto_punctuation,
    theme,
    onChange: (v) => { config.post_processing.auto_punctuation = v; markDirty() },
  })

  const ppCapitalize = createToggle(renderer, {
    id: "pp-caps",
    label: "Auto-capitalize",
    value: config.post_processing.auto_capitalize,
    theme,
    onChange: (v) => { config.post_processing.auto_capitalize = v; markDirty() },
  })

  const ppFiller = createToggle(renderer, {
    id: "pp-filler",
    label: "Remove filler words",
    value: config.post_processing.remove_filler_words,
    theme,
    onChange: (v) => { config.post_processing.remove_filler_words = v; markDirty() },
  })

  ppContent.add(ppPunctuation.root)
  ppContent.add(ppCapitalize.root)
  ppContent.add(ppFiller.root)

  // ── 7. Injection ───────────────────────────────────────────────
  const injContent = createSection("sec-injection", "Text Injection")

  const injMethodLabel = new TextRenderable(renderer, {
    id: "inj-method-label",
    content: "Method:",
    fg: theme.colors.textSecondary,
  })
  const injMethodLabelBox = new BoxRenderable(renderer, {
    id: "inj-method-label-box",
    width: "100%" as any,
  })
  injMethodLabelBox.add(injMethodLabel)
  
  const injMethodSelect = new SelectRenderable(renderer, {
    id: "inj-method-select",
    width: 35,
    height: 4,
    options: [
      { name: "accessibility", description: "OS accessibility API (recommended)" },
      { name: "clipboard", description: "Copy to clipboard" },
      { name: "paste", description: "Simulate paste" },
    ],
    backgroundColor: RGBA.fromHex(theme.colors.surface),
    focusedBackgroundColor: RGBA.fromHex(theme.colors.hover),
    focusedTextColor: RGBA.fromHex(theme.colors.textPrimary),
    selectedBackgroundColor: RGBA.fromHex(theme.colors.selected),
    textColor: RGBA.fromHex(theme.colors.textSecondary),
    showDescription: true,
  })
  const injIdx = ["accessibility", "clipboard", "paste"].indexOf(config.injection.method)
  if (injIdx >= 0) injMethodSelect.setSelectedIndex(injIdx)
  injMethodSelect.on(SelectRenderableEvents.SELECTION_CHANGED, (index: number) => {
    config.injection.method = ["accessibility", "clipboard", "paste"][index]
    markDirty()
  })

  const DELAY_VALUES = ["0", "10", "20", "30", "50", "75", "100", "150", "200", "300", "500"]
  const injDelayStepper = createStepper(renderer, {
    id: "inj-delay",
    label: "Paste Delay (ms):",
    values: DELAY_VALUES,
    selectedIndex: Math.max(0, DELAY_VALUES.indexOf(config.injection.paste_delay_ms.toString())),
    theme,
    onChange: (v) => { config.injection.paste_delay_ms = parseInt(v, 10); markDirty() },
  })

  injContent.add(injMethodLabelBox)
  injContent.add(injMethodSelect)
  injContent.add(injDelayStepper.root)

  // ── 8. History ─────────────────────────────────────────────────
  const histContent = createSection("sec-history", "History Settings")

  const histEnabled = createToggle(renderer, {
    id: "hist-enabled",
    label: "Record history",
    value: config.history.enabled,
    theme,
    onChange: (v) => { config.history.enabled = v; markDirty() },
  })

  const MAX_ENTRIES_VALUES = ["100", "200", "500", "1000", "2000", "5000", "10000"]
  const histMaxStepper = createStepper(renderer, {
    id: "hist-max",
    label: "Max entries:",
    values: MAX_ENTRIES_VALUES,
    selectedIndex: Math.max(0, MAX_ENTRIES_VALUES.indexOf(config.history.max_entries.toString())),
    theme,
    onChange: (v) => { config.history.max_entries = parseInt(v, 10); markDirty() },
  })

  histContent.add(histEnabled.root)
  histContent.add(histMaxStepper.root)

  // ── (sections are already added to scrollBox via createSection) ────

  // ── Focus management ─────────────────────────────────────────────────
  // scrollHint = approximate top row to scroll to so the widget is visible
  type FocusItem =
    | { type: "select";     widget: SelectRenderable;                   scrollHint: number }
    | { type: "stepper";    instance: StepperInstance;                  scrollHint: number }
    | { type: "toggle";     instance: ReturnType<typeof createToggle>;  scrollHint: number }
    | { type: "keycapture"; instance: KeyCaptureInstance;               scrollHint: number }

  // Populated after all widget declarations; deviceSelect spliced in async
  // scrollHints are approximate terminal-row offsets for each widget
  let focusables: FocusItem[] = [
    { type: "select",     widget: modelSelect,          scrollHint: 0  },
    { type: "keycapture", instance: triggerCapture,     scrollHint: 11 },
    { type: "keycapture", instance: toggleCapture,      scrollHint: 12 },
    { type: "select",     widget: modeSelect,           scrollHint: 14 },
    // index 4 reserved for deviceSelect (inserted asynchronously → scrollHint 20)
    { type: "stepper",    instance: srStepper,          scrollHint: 28 },
    { type: "stepper",    instance: chunkStepper,       scrollHint: 29 },
    { type: "toggle",     instance: vadEnabled,         scrollHint: 34 },
    { type: "select",     widget: vadBackendSelect,     scrollHint: 36 },
    { type: "stepper",    instance: vadThresholdStepper, scrollHint: 42 },
    { type: "toggle",     instance: vadAdaptive,        scrollHint: 43 },
    { type: "toggle",     instance: ppPunctuation,      scrollHint: 50 },
    { type: "toggle",     instance: ppCapitalize,       scrollHint: 51 },
    { type: "toggle",     instance: ppFiller,           scrollHint: 52 },
    { type: "select",     widget: injMethodSelect,      scrollHint: 59 },
    { type: "stepper",    instance: injDelayStepper,    scrollHint: 64 },
    { type: "toggle",     instance: histEnabled,        scrollHint: 72 },
    { type: "stepper",    instance: histMaxStepper,     scrollHint: 73 },
  ]

  let focusedIdx = -1

  function blurCurrent() {
    if (focusedIdx < 0) return
    const cur = focusables[focusedIdx]
    if (cur.type === "toggle")     cur.instance.blur()
    if (cur.type === "keycapture") cur.instance.blur()
    if (cur.type === "stepper")    cur.instance.blur()
  }

  function applyFocus(idx: number) {
    blurCurrent()
    focusedIdx = idx
    const item = focusables[idx]
    // Scroll to approximate position of the focused widget
    scrollBox.scrollTop = item.scrollHint
    if (item.type === "select") item.widget.focus()
    else {
      // Non-Renderable-focusable widgets: clear renderer focus so arrows
      // don't accidentally route to a previously-focused select.
      ;(renderer as any).focusRenderable(null)
      item.instance.focus()
    }
  }

  function focusFirst() {
    scrollBox.scrollTop = 0
    if (focusables.length > 0) applyFocus(0)
  }
  function focusNext()  { applyFocus((focusedIdx + 1 + focusables.length) % focusables.length) }
  function focusPrev()  { applyFocus((focusedIdx - 1 + focusables.length) % focusables.length) }
  function blurAll()    { blurCurrent(); focusedIdx = -1 }
  function hasFocus()   { return focusedIdx >= 0 }

  // Intercepts Tab / Shift-Tab / Escape / Space-for-toggle / Left-Right-for-stepper
  // before keypress events fire so navigation keys don't bleed into the tab-bar handler.
  const configInputHandler = (seq: string): boolean => {
    if (state.activeTab !== 1 || focusedIdx < 0) return false
    const cur = focusables[focusedIdx]
    // When a key-capture widget is active, only intercept Tab/Shift-Tab/Escape;
    // everything else must reach the keypress listener it registered.
    if (cur?.type === "keycapture") {
      if (seq === "\t")      { cur.instance.blur(); focusNext(); return true }
      if (seq === "\x1b[Z") { cur.instance.blur(); focusPrev(); return true }
      if (seq === "\x1b")   { cur.instance.blur(); blurAll(); callbacks.onEscape?.(); return true }
      return false // let the keypress event through to key-capture's own handler
    }
    if (seq === "\t")      { focusNext(); return true }
    if (seq === "\x1b[Z") { focusPrev(); return true }  // Shift+Tab
    if (seq === "\x1b")   { blurAll(); callbacks.onEscape?.(); return true }
    if (seq === " ")       { if (cur?.type === "toggle")  { cur.instance.toggle(); return true } }
    if (seq === "\x1b[D") { if (cur?.type === "stepper") { cur.instance.prev();   return true } } // Left
    if (seq === "\x1b[C") { if (cur?.type === "stepper") { cur.instance.next();   return true } } // Right
    return false
  }
  renderer.prependInputHandler(configInputHandler)

  // ── Save function ────────────────────────────────────────────────────
  function save() {
    try {
      saveConfig(config)
      callbacks.onSaved()
    } catch (e) {
      callbacks.onStatusMessage(`✗ Failed to save: ${e}`)
    }
  }

  function destroy() {
    blurAll()
    renderer.removeInputHandler(configInputHandler)
    triggerCapture.destroy()
    toggleCapture.destroy()
  }

  return { root, save, focusFirst, blurAll, hasFocus, destroy }
}
