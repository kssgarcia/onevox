/**
 * Config Panel — flat minimalist settings, always-visible sections.
 *
 * Sections:
 *   1. Daemon
 *   2. Model Selection
 *   3. Key Bindings
 *   4. Device Selection
 *   5. Audio Settings
 *   6. VAD Settings
 *   7. Post Processing
 *   8. Injection
 *   9. UI
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
import { saveConfig, type VoxConfig } from "../data/config.js"
import { getModelRegistry, listDevicesWithError, reloadDaemonConfig } from "../data/cli.js"
import { createToggle, type ToggleInstance } from "../components/toggle.js"
import { createKeyCapture, type KeyCaptureInstance } from "../components/key-capture.js"
import { createStepper, type StepperInstance } from "../components/stepper.js"
import { createSelectField, type SelectFieldInstance } from "../components/select-field.js"

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

  // ── 1. Daemon ──────────────────────────────────────────────────────
  const daemonContent = createSection("sec-daemon", "Daemon")

  const daemonAutoStart = createToggle(renderer, {
    id: "daemon-auto-start",
    label: "Auto-start daemon",
    value: config.daemon.auto_start,
    theme,
    onChange: (v) => { config.daemon.auto_start = v; markDirty() },
  })

  const logIdx = ["trace", "debug", "info", "warn", "error"].indexOf(config.daemon.log_level)
  const daemonLogLevelField = createSelectField(renderer, {
    id: "daemon-log-level",
    label: "Log Level:",
    options: [
      { name: "trace", description: "Most verbose logging" },
      { name: "debug", description: "Debug + info logs" },
      { name: "info", description: "General operational logs" },
      { name: "warn", description: "Warnings and errors only" },
      { name: "error", description: "Errors only" },
    ],
    selectedIndex: logIdx >= 0 ? logIdx : 2,
    theme,
    onChange: (index) => {
      config.daemon.log_level = ["trace", "debug", "info", "warn", "error"][index]
      markDirty()
    },
  })

  daemonContent.add(daemonAutoStart.root)
  daemonContent.add(daemonLogLevelField.root)

  // ── 2. Model Selection ─────────────────────────────────────────────
  const modelContent = createSection("sec-model", "Model Selection")

  const models = getModelRegistry()

  // Pre-select current model
  const currentModelIdx = models.findIndex((m) => {
    const current = config.model.model_path
    return (
      current === m.id ||
      current === `${m.id}.bin` ||
      current.includes(m.id) ||
      current.includes(`${m.id}.bin`)
    )
  })
  const modelField = createSelectField(renderer, {
    id: "model-select",
    label: "Model:",
    options: models.map((m) => ({
      name: `${m.name}  (${m.size})`,
      description: `${m.speedFactor}x speed  •  ${m.memoryMb}MB RAM  •  ${m.description}`,
    })),
    selectedIndex: currentModelIdx >= 0 ? currentModelIdx : 0,
    theme,
    onChange: (index) => {
      config.model.model_path = `${models[index].id}.bin`
      markDirty()
      callbacks.onStatusMessage(`Model → ${models[index].name}`)
      setTimeout(() => callbacks.onStatusMessage(""), 1500)
    },
  })

  modelContent.add(modelField.root)

  const modelBackendIdx = ["whisper_cpp", "faster_whisper", "onnx", "candle"].indexOf(config.model.backend)
  const modelBackendField = createSelectField(renderer, {
    id: "model-backend-select",
    label: "Backend:",
    options: [
      { name: "whisper_cpp", description: "whisper.cpp CLI backend" },
      { name: "faster_whisper", description: "CTranslate2 backend" },
      { name: "onnx", description: "ONNX Runtime backend" },
      { name: "candle", description: "Rust Candle backend" },
    ],
    selectedIndex: modelBackendIdx >= 0 ? modelBackendIdx : 0,
    theme,
    onChange: (index) => {
      config.model.backend = ["whisper_cpp", "faster_whisper", "onnx", "candle"][index]
      markDirty()
    },
  })

  const modelDeviceIdx = ["auto", "cpu", "gpu"].indexOf(config.model.device)
  const modelDeviceField = createSelectField(renderer, {
    id: "model-device-select",
    label: "Model Device:",
    options: [
      { name: "auto", description: "Automatic device selection" },
      { name: "cpu", description: "CPU only" },
      { name: "gpu", description: "GPU acceleration if available" },
    ],
    selectedIndex: modelDeviceIdx >= 0 ? modelDeviceIdx : 0,
    theme,
    onChange: (index) => {
      config.model.device = ["auto", "cpu", "gpu"][index]
      markDirty()
    },
  })

  const modelLanguageIdx = ["en", "auto"].indexOf(config.model.language)
  const modelLanguageField = createSelectField(renderer, {
    id: "model-language-select",
    label: "Language:",
    options: [
      { name: "en", description: "English" },
      { name: "auto", description: "Auto-detect language" },
    ],
    selectedIndex: modelLanguageIdx >= 0 ? modelLanguageIdx : 0,
    theme,
    onChange: (index) => {
      config.model.language = ["en", "auto"][index]
      markDirty()
    },
  })

  const modelTaskIdx = ["transcribe", "translate"].indexOf(config.model.task)
  const modelTaskField = createSelectField(renderer, {
    id: "model-task-select",
    label: "Task:",
    options: [
      { name: "transcribe", description: "Speech to source language text" },
      { name: "translate", description: "Speech translated to English" },
    ],
    selectedIndex: modelTaskIdx >= 0 ? modelTaskIdx : 0,
    theme,
    onChange: (index) => {
      config.model.task = ["transcribe", "translate"][index]
      markDirty()
    },
  })

  const modelPreload = createToggle(renderer, {
    id: "model-preload",
    label: "Preload model at startup",
    value: config.model.preload,
    theme,
    onChange: (v) => { config.model.preload = v; markDirty() },
  })

  modelContent.add(modelBackendField.root)
  modelContent.add(modelDeviceField.root)
  modelContent.add(modelLanguageField.root)
  modelContent.add(modelTaskField.root)
  modelContent.add(modelPreload.root)

  // ── 3. Key Bindings ────────────────────────────────────────────────
  const hotkeyContent = createSection("sec-hotkey", "Key Bindings")

  // Push-to-talk trigger
  const triggerCapture = createKeyCapture(renderer, {
    id: "trigger-capture",
    label: "Push-to-talk trigger:",
    value: config.hotkey.trigger,
    theme,
    onChange: (combo) => { config.hotkey.trigger = combo; markDirty() },
  })

  const modeIdx = config.hotkey.mode === "toggle" ? 1 : 0
  const modeField = createSelectField(renderer, {
    id: "mode-select",
    label: "Hotkey Mode:",
    options: [
      { name: "push-to-talk", description: "Hold key to dictate" },
      { name: "toggle", description: "Press to start/stop" },
    ],
    selectedIndex: modeIdx,
    theme,
    onChange: (index) => {
      config.hotkey.mode = index === 0 ? "push-to-talk" : "toggle"
      markDirty()
    },
  })

  hotkeyContent.add(triggerCapture.root)
  hotkeyContent.add(modeField.root)

  // ── 4. Device Selection ────────────────────────────────────────
  const deviceContent = createSection("sec-device", "Device Selection")

  const deviceLoading = new TextRenderable(renderer, {
    id: "device-loading",
    content: "Loading devices...",
    fg: theme.colors.textMuted,
  })
  deviceContent.add(deviceLoading)

  // Async load devices
  let deviceFieldRef: SelectFieldInstance | null = null
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
        content: error ? "Could not list devices — check that `onevox` is built" : "No audio input devices found",
        fg: theme.colors.textMuted,
      })
      deviceContent.add(noDevices)
      return
    }

    // Pre-select current device
    const curIdx = devices.findIndex(
      (d) => d.name === config.audio.device || (config.audio.device === "default" && d.isDefault),
    )
    const deviceField = createSelectField(renderer, {
      id: "device-select",
      label: "Input Device:",
      options: devices.map((d) => ({
        name: `${d.name}${d.isDefault ? " (default)" : ""}`,
        description: `${d.sampleRate}Hz, ${d.channels}ch`,
      })),
      selectedIndex: curIdx >= 0 ? curIdx : 0,
      theme,
      onChange: (index) => {
        config.audio.device = devices[index].name
        markDirty()
        callbacks.onStatusMessage(`Device → ${devices[index].name}`)
        setTimeout(() => callbacks.onStatusMessage(""), 1500)
      },
    })
    deviceFieldRef = deviceField

    deviceContent.add(deviceField.root)
    // Register in keyboard focus navigation (inserted before srStepper at index 10)
    focusables.splice(10, 0, { type: "selectfield", instance: deviceField, scrollHint: 22 })
    bindMouseFocusHandlers()
  })

  // ── 5. Audio Settings ──────────────────────────────────────────
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

  // ── 6. VAD Settings ────────────────────────────────────────────
  const vadContent = createSection("sec-vad", "VAD (Voice Activity Detection)")

  const vadEnabled = createToggle(renderer, {
    id: "vad-enabled",
    label: "Enabled",
    value: config.vad.enabled,
    theme,
    onChange: (v) => { config.vad.enabled = v; markDirty() },
  })

  const vadIdx = ["energy", "silero", "webrtc"].indexOf(config.vad.backend)
  const vadBackendField = createSelectField(renderer, {
    id: "vad-backend-select",
    label: "Backend:",
    options: [
      { name: "energy", description: "Simple energy-based detection" },
      { name: "silero", description: "Neural network-based (more accurate)" },
      { name: "webrtc", description: "WebRTC VAD library" },
    ],
    selectedIndex: vadIdx >= 0 ? vadIdx : 0,
    theme,
    onChange: (index) => {
      config.vad.backend = ["energy", "silero", "webrtc"][index]
      markDirty()
    },
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

  const PRE_POST_VALUES = ["0", "100", "200", "300", "500", "750", "1000", "1500", "2000"]
  const vadPreRollStepper = createStepper(renderer, {
    id: "vad-pre-roll",
    label: "Pre-roll (ms):",
    values: PRE_POST_VALUES,
    selectedIndex: Math.max(0, PRE_POST_VALUES.indexOf(config.vad.pre_roll_ms.toString())),
    theme,
    onChange: (v) => { config.vad.pre_roll_ms = parseInt(v, 10); markDirty() },
  })

  const vadPostRollStepper = createStepper(renderer, {
    id: "vad-post-roll",
    label: "Post-roll (ms):",
    values: PRE_POST_VALUES,
    selectedIndex: Math.max(0, PRE_POST_VALUES.indexOf(config.vad.post_roll_ms.toString())),
    theme,
    onChange: (v) => { config.vad.post_roll_ms = parseInt(v, 10); markDirty() },
  })

  const MIN_CHUNK_VALUES = ["1", "2", "3", "4", "5", "6", "8", "10"]
  const vadMinSpeechStepper = createStepper(renderer, {
    id: "vad-min-speech",
    label: "Min speech chunks:",
    values: MIN_CHUNK_VALUES,
    selectedIndex: Math.max(0, MIN_CHUNK_VALUES.indexOf(config.vad.min_speech_chunks.toString())),
    theme,
    onChange: (v) => { config.vad.min_speech_chunks = parseInt(v, 10); markDirty() },
  })

  const vadMinSilenceStepper = createStepper(renderer, {
    id: "vad-min-silence",
    label: "Min silence chunks:",
    values: MIN_CHUNK_VALUES,
    selectedIndex: Math.max(0, MIN_CHUNK_VALUES.indexOf(config.vad.min_silence_chunks.toString())),
    theme,
    onChange: (v) => { config.vad.min_silence_chunks = parseInt(v, 10); markDirty() },
  })

  vadContent.add(vadEnabled.root)
  vadContent.add(vadBackendField.root)
  vadContent.add(vadThresholdStepper.root)
  vadContent.add(vadAdaptive.root)
  vadContent.add(vadPreRollStepper.root)
  vadContent.add(vadPostRollStepper.root)
  vadContent.add(vadMinSpeechStepper.root)
  vadContent.add(vadMinSilenceStepper.root)

  // ── 7. Post Processing ─────────────────────────────────────────
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

  // ── 8. Injection ───────────────────────────────────────────────
  const injContent = createSection("sec-injection", "Text Injection")

  const injIdx = ["accessibility", "clipboard", "paste"].indexOf(config.injection.method)
  const injMethodField = createSelectField(renderer, {
    id: "inj-method-select",
    label: "Method:",
    options: [
      { name: "accessibility", description: "OS accessibility API (recommended)" },
      { name: "clipboard", description: "Copy to clipboard" },
      { name: "paste", description: "Simulate paste" },
    ],
    selectedIndex: injIdx >= 0 ? injIdx : 0,
    theme,
    onChange: (index) => {
      config.injection.method = ["accessibility", "clipboard", "paste"][index]
      markDirty()
    },
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

  const FOCUS_SETTLE_VALUES = ["0", "20", "40", "60", "80", "100", "120", "150", "200", "300"]
  const injFocusSettleStepper = createStepper(renderer, {
    id: "inj-focus-settle",
    label: "Focus settle (ms):",
    values: FOCUS_SETTLE_VALUES,
    selectedIndex: Math.max(
      0,
      FOCUS_SETTLE_VALUES.indexOf(config.injection.focus_settle_ms.toString()),
    ),
    theme,
    onChange: (v) => { config.injection.focus_settle_ms = parseInt(v, 10); markDirty() },
  })

  injContent.add(injMethodField.root)
  injContent.add(injDelayStepper.root)
  injContent.add(injFocusSettleStepper.root)

  // ── 9. UI ──────────────────────────────────────────────────────
  const uiContent = createSection("sec-ui", "UI")

  const uiOverlayToggle = createToggle(renderer, {
    id: "ui-recording-overlay",
    label: "Recording overlay indicator",
    value: config.ui.recording_overlay,
    theme,
    onChange: (v) => { config.ui.recording_overlay = v; markDirty() },
  })

  uiContent.add(uiOverlayToggle.root)

  // ── (sections are already added to scrollBox via createSection) ────

  // ── Focus management ─────────────────────────────────────────────────
  // scrollHint = approximate top row to scroll to so the widget is visible
  type FocusItem =
    | { type: "selectfield"; instance: SelectFieldInstance;             scrollHint: number }
    | { type: "stepper";    instance: StepperInstance;                  scrollHint: number }
    | { type: "toggle";     instance: ReturnType<typeof createToggle>;  scrollHint: number }
    | { type: "keycapture"; instance: KeyCaptureInstance;               scrollHint: number }

  // Populated after all widget declarations; deviceSelect spliced in async
  // scrollHints are approximate terminal-row offsets for each widget
  let focusables: FocusItem[] = [
    { type: "toggle",     instance: daemonAutoStart,      scrollHint: 0 },
    { type: "selectfield", instance: daemonLogLevelField,  scrollHint: 2 },
    { type: "selectfield", instance: modelField,           scrollHint: 6 },
    { type: "selectfield", instance: modelBackendField,    scrollHint: 8 },
    { type: "selectfield", instance: modelDeviceField,     scrollHint: 9 },
    { type: "selectfield", instance: modelLanguageField,   scrollHint: 10 },
    { type: "selectfield", instance: modelTaskField,       scrollHint: 11 },
    { type: "toggle",     instance: modelPreload,          scrollHint: 12 },
    { type: "keycapture", instance: triggerCapture,        scrollHint: 16 },
    { type: "selectfield", instance: modeField,            scrollHint: 17 },
    // index 10 reserved for deviceSelect (inserted asynchronously → scrollHint 22)
    { type: "stepper",    instance: srStepper,             scrollHint: 26 },
    { type: "stepper",    instance: chunkStepper,          scrollHint: 27 },
    { type: "toggle",     instance: vadEnabled,            scrollHint: 32 },
    { type: "selectfield", instance: vadBackendField,      scrollHint: 33 },
    { type: "stepper",    instance: vadThresholdStepper,   scrollHint: 34 },
    { type: "toggle",     instance: vadAdaptive,           scrollHint: 35 },
    { type: "stepper",    instance: vadPreRollStepper,     scrollHint: 36 },
    { type: "stepper",    instance: vadPostRollStepper,    scrollHint: 37 },
    { type: "stepper",    instance: vadMinSpeechStepper,   scrollHint: 38 },
    { type: "stepper",    instance: vadMinSilenceStepper,  scrollHint: 39 },
    { type: "toggle",     instance: ppPunctuation,         scrollHint: 44 },
    { type: "toggle",     instance: ppCapitalize,          scrollHint: 45 },
    { type: "toggle",     instance: ppFiller,              scrollHint: 46 },
    { type: "selectfield", instance: injMethodField,       scrollHint: 51 },
    { type: "stepper",    instance: injDelayStepper,       scrollHint: 52 },
    { type: "stepper",    instance: injFocusSettleStepper, scrollHint: 53 },
    { type: "toggle",     instance: uiOverlayToggle,       scrollHint: 58 },
  ]

  let focusedIdx = -1

  function blurCurrent() {
    if (focusedIdx < 0) return
    const cur = focusables[focusedIdx]
    if (cur.type === "selectfield") cur.instance.blur()
    if (cur.type === "toggle")     cur.instance.blur()
    if (cur.type === "keycapture") cur.instance.blur()
    if (cur.type === "stepper")    cur.instance.blur()
  }

  function getFocusRoot(item: FocusItem) {
    if (item.type === "selectfield") return item.instance.root as any
    if (item.type === "stepper") return item.instance.root as any
    if (item.type === "toggle") return item.instance.root as any
    return item.instance.root as any
  }

  function ensureFocusedVisible(item: FocusItem) {
    const node = getFocusRoot(item)
    const itemTop = Number(node?.y ?? 0)
    const itemHeight = Math.max(1, Number(node?.height ?? 1))
    const itemBottom = itemTop + itemHeight - 1

    const currentTop = Number(scrollBox.scrollTop || 0)
    const viewportTop = Number((scrollBox as any).viewport?.y ?? 0)
    const viewportRows = Math.max(
      1,
      Number((scrollBox as any).viewportHeight ?? (scrollBox as any).viewport?.height ?? 20),
    )
    const viewportBottom = viewportTop + viewportRows - 1

    if (itemTop < viewportTop) {
      scrollBox.scrollTop = Math.max(0, currentTop - (viewportTop - itemTop))
      return
    }
    if (itemBottom > viewportBottom) {
      scrollBox.scrollTop = Math.max(0, currentTop + (itemBottom - viewportBottom))
    }
  }

  function applyFocus(idx: number) {
    if (idx < 0 || idx >= focusables.length) return
    blurCurrent()
    focusedIdx = idx
    const item = focusables[idx]
    // Scroll only when focused widget leaves viewport
    ensureFocusedVisible(item)
    if (item.type === "selectfield") item.instance.focus()
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
  function focusNext()  { applyFocus(Math.min(focusedIdx + 1, focusables.length - 1)) }
  function focusPrev()  { applyFocus(Math.max(focusedIdx - 1, 0)) }
  function blurAll()    { blurCurrent(); focusedIdx = -1 }
  function hasFocus()   { return focusedIdx >= 0 }

  function bindMouseFocusHandlers() {
    for (let i = 0; i < focusables.length; i++) {
      const item = focusables[i]
      const rootNode =
        item.type === "selectfield" ? item.instance.root :
        item.type === "stepper" ? item.instance.root :
        item.type === "toggle" ? item.instance.root :
        item.instance.root
      if ((rootNode as any).__focusBound) continue
      const prevMouseDown = (rootNode as any).onMouseDown
      ;(rootNode as any).onMouseDown = (...args: any[]) => {
        const idx = focusables.findIndex((f) => {
          if (f.type === "selectfield") return f.instance.root === rootNode
          if (f.type === "stepper") return f.instance.root === rootNode
          if (f.type === "toggle") return f.instance.root === rootNode
          return f.instance.root === rootNode
        })
        if (idx >= 0) applyFocus(idx)
        if (prevMouseDown) return prevMouseDown(...args)
      }
      ;(rootNode as any).__focusBound = true
    }
  }
  bindMouseFocusHandlers()

  // Intercepts arrow keys / hjkl / Enter / Escape / Space-for-toggle / Left-Right-for-stepper
  // before keypress events fire so navigation keys don't bleed into the tab-bar handler.
  const configInputHandler = (seq: string): boolean => {
    if (state.activeTab !== 1 || focusedIdx < 0) return false
    const cur = focusables[focusedIdx]
    
    // When focused on a key-capture widget:
    // - while popup capture is open, let non-nav keys reach key-capture listener.
    // - while popup is closed, only intercept nav/open keys; let others propagate.
    if (cur?.type === "keycapture") {
      // While popup capture is open:
      // - Esc should cancel capture immediately.
      // - Ctrl+C must propagate so the app can exit.
      // - Other keys should pass through to key-capture's keypress listener.
      if (cur.instance.isCapturing()) {
        if (seq === "\x1b") { cur.instance.cancelCapture(); return true }
        if (seq === "\x03") return false
        return false
      }

      // Down/j or Up/k to navigate
      if (seq === "\x1b[B" || seq === "j") { cur.instance.blur(); focusNext(); return true }  // Down
      if (seq === "\x1b[A" || seq === "k") { cur.instance.blur(); focusPrev(); return true }  // Up
      if (seq === "\x1b")                   { cur.instance.blur(); blurAll(); callbacks.onEscape?.(); return true }  // Escape
      if (seq === "\r" || seq === "\n" || seq === " ") { cur.instance.open(); return true } // Enter/Space opens popup
      return false
    }
    
    // Down/j: navigate to next widget
    if (seq === "\x1b[B" || seq === "j") { focusNext(); return true }
    // Up/k: navigate to previous widget
    if (seq === "\x1b[A" || seq === "k") { focusPrev(); return true }
    // Escape: blur all and return to tabs
    if (seq === "\x1b")                   { blurAll(); callbacks.onEscape?.(); return true }
    // Space/Enter: toggle toggles
    if (seq === " " || seq === "\r" || seq === "\n") {
      if (cur?.type === "toggle")  { cur.instance.toggle(); return true }
    }
    // Enter/Space: open selection popup
    if (seq === " " || seq === "\r" || seq === "\n") {
      if (cur?.type === "selectfield") { cur.instance.open(); return true }
    }
    // Left/h: previous value for stepper
    if (seq === "\x1b[D" || seq === "h") { if (cur?.type === "stepper") { cur.instance.prev();   return true } }
    // Right/l: next value for stepper
    if (seq === "\x1b[C" || seq === "l") { if (cur?.type === "stepper") { cur.instance.next();   return true } }

    return false
  }
  renderer.prependInputHandler(configInputHandler)

  // ── Save function ────────────────────────────────────────────────────
  function save() {
    try {
      saveConfig(config)
      callbacks.onSaved()
      void reloadDaemonConfig().then((result) => {
        if (result.state === "reloaded") {
          callbacks.onStatusMessage("✓ Saved and applied to daemon")
          return
        }
        if (result.state === "not_running") {
          callbacks.onStatusMessage("✓ Saved (daemon not running)")
          return
        }
        callbacks.onStatusMessage(`✓ Saved (reload failed: ${result.message})`)
      })
    } catch (e) {
      callbacks.onStatusMessage(`✗ Failed to save: ${e}`)
    }
  }

  function destroy() {
    blurAll()
    renderer.removeInputHandler(configInputHandler)
    daemonLogLevelField.destroy()
    modelField.destroy()
    modelBackendField.destroy()
    modelDeviceField.destroy()
    modelLanguageField.destroy()
    modelTaskField.destroy()
    modeField.destroy()
    vadBackendField.destroy()
    injMethodField.destroy()
    deviceFieldRef?.destroy()
    triggerCapture.destroy()
  }

  return { root, save, focusFirst, blurAll, hasFocus, destroy }
}
