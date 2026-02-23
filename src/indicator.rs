//! Desktop recording/processing overlay indicator.

use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorMode {
    Recording,
    Processing,
}

impl IndicatorMode {
    pub fn from_cli(value: &str) -> Option<Self> {
        match value {
            "recording" => Some(Self::Recording),
            "processing" => Some(Self::Processing),
            _ => None,
        }
    }

    fn as_cli(self) -> &'static str {
        match self {
            Self::Recording => "recording",
            Self::Processing => "processing",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Recording => "RECORDING",
            Self::Processing => "PROCESSING",
        }
    }

    fn amplitude(self) -> f32 {
        match self {
            Self::Recording => 1.0,
            Self::Processing => 0.6,
        }
    }
}

struct ChildIndicator {
    child: Child,
}

#[derive(Default)]
struct IndicatorRuntime {
    child: Option<ChildIndicator>,
}

/// Cross-platform indicator controller.
///
/// Uses a tiny child process for the overlay so macOS can create the window
/// event loop on that process main thread.
pub struct RecordingIndicator {
    enabled: bool,
    runtime: Mutex<IndicatorRuntime>,
}

impl RecordingIndicator {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: enabled && cfg!(feature = "overlay-indicator"),
            runtime: Mutex::new(IndicatorRuntime::default()),
        }
    }

    pub fn recording(&self) {
        self.show(IndicatorMode::Recording);
    }

    pub fn processing(&self) {
        self.show(IndicatorMode::Processing);
    }

    pub fn hide(&self) {
        if !self.enabled {
            return;
        }

        write_indicator_state(None);

        let mut guard = match self.runtime.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        stop_child(&mut guard.child);
    }

    fn show(&self, mode: IndicatorMode) {
        if !self.enabled {
            return;
        }

        write_indicator_state(Some(mode));

        let mut guard = match self.runtime.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        if let Some(existing) = &mut guard.child {
            if let Ok(None) = existing.child.try_wait() {
                return;
            }
            stop_child(&mut guard.child);
        }

        if let Some(child) = spawn_child(mode) {
            guard.child = Some(ChildIndicator { child });
        } else {
            tracing::warn!("Failed to start overlay indicator process");
        }
    }
}

fn stop_child(slot: &mut Option<ChildIndicator>) {
    if let Some(mut child) = slot.take() {
        let _ = child.child.kill();
        let _ = child.child.wait();
    }
}

fn spawn_child(mode: IndicatorMode) -> Option<Child> {
    let exe = std::env::current_exe().ok()?;
    Command::new(exe)
        .arg("indicator")
        .arg("--mode")
        .arg(mode.as_cli())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

fn indicator_state_path() -> Option<PathBuf> {
    crate::platform::paths::cache_dir()
        .ok()
        .map(|d| d.join("indicator.state"))
}

fn write_indicator_state(mode: Option<IndicatorMode>) {
    let Some(path) = indicator_state_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let value = match mode {
        Some(IndicatorMode::Recording) => "recording",
        Some(IndicatorMode::Processing) => "processing",
        None => "hidden",
    };
    let _ = fs::write(path, value);
}

fn read_indicator_state() -> Option<Option<IndicatorMode>> {
    let path = indicator_state_path()?;
    let content = fs::read_to_string(path).ok()?;
    let value = content.trim();
    let parsed = match value {
        "recording" => Some(IndicatorMode::Recording),
        "processing" => Some(IndicatorMode::Processing),
        "hidden" => None,
        _ => return None,
    };
    Some(parsed)
}

/// Run overlay UI process.
///
/// This must execute on the process main thread.
pub fn run_indicator(mode: IndicatorMode) -> crate::Result<()> {
    #[cfg(not(feature = "overlay-indicator"))]
    {
        let _ = mode;
        return Ok(());
    }

    #[cfg(feature = "overlay-indicator")]
    {
        use eframe::egui;
        use std::time::{Duration, Instant};
        const WINDOW_WIDTH: f32 = 110.0;
        const WINDOW_HEIGHT: f32 = 36.0;
        const BOTTOM_MARGIN: f32 = 20.0;

        struct OverlayApp {
            mode: IndicatorMode,
            phase_start: Instant,
            positioned: bool,
            last_state_poll: Instant,
            frozen_phase: f32,
        }

        impl OverlayApp {
            fn draw_waveform(&self, ui: &mut egui::Ui, t: f32) {
                let desired = egui::vec2(ui.available_width(), ui.available_height());
                let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
                let painter = ui.painter_at(rect);

                let center_y = rect.center().y;
                let left = rect.left();
                let width = rect.width();
                let lane_count = 3usize;
                let points_per_lane = 70usize;

                for lane in 0..lane_count {
                    let lane_offset = (lane as f32 - 1.0) * 3.0;
                    let lane_phase = t * 3.6 + lane as f32 * 0.65;
                    let amplitude = self.mode.amplitude();
                    let mut points = Vec::with_capacity(points_per_lane);

                    for i in 0..points_per_lane {
                        let x_norm = i as f32 / (points_per_lane as f32 - 1.0);
                        let x = left + x_norm * width;
                        let envelope = (-18.0 * (x_norm - 0.5).powi(2)).exp();
                        let signal = (x_norm * std::f32::consts::TAU * 2.0 + lane_phase).sin()
                            + 0.45
                                * (x_norm * std::f32::consts::TAU * 4.5 - lane_phase * 1.5).sin();
                        let y = center_y + lane_offset + signal * envelope * 4.5 * amplitude;
                        points.push(egui::pos2(x, y));
                    }

                    painter.add(egui::Shape::line(
                        points,
                        egui::Stroke::new(1.2, egui::Color32::WHITE),
                    ));
                }
            }
        }

        impl eframe::App for OverlayApp {
            fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
                if self.last_state_poll.elapsed() >= Duration::from_millis(60) {
                    self.last_state_poll = Instant::now();
                    if let Some(state) = read_indicator_state() {
                        match state {
                            Some(mode) => {
                                if mode != self.mode {
                                    // Mode changed - freeze the phase if switching to Processing
                                    if mode == IndicatorMode::Processing {
                                        self.frozen_phase =
                                            self.phase_start.elapsed().as_secs_f32();
                                    }
                                    self.mode = mode;
                                }
                            }
                            None => {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                return;
                            }
                        }
                    }
                }

                // Use frozen phase for processing mode, live elapsed time for recording
                let elapsed = match self.mode {
                    IndicatorMode::Recording => self.phase_start.elapsed().as_secs_f32(),
                    IndicatorMode::Processing => self.frozen_phase,
                };

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(egui::Color32::BLACK)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(0.0)
                            .inner_margin(egui::Margin::same(4))
                            .outer_margin(egui::Margin::ZERO),
                    )
                    .show(ctx, |ui| {
                        if !self.positioned
                            && let Some(size) = ctx.input(|i| i.viewport().monitor_size)
                        {
                            let x = ((size.x - WINDOW_WIDTH) * 0.5).max(0.0);
                            let y = (size.y - WINDOW_HEIGHT - BOTTOM_MARGIN).max(0.0);
                            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                                egui::pos2(x, y),
                            ));
                            self.positioned = true;
                        }

                        self.draw_waveform(ui, elapsed);
                    });

                ctx.request_repaint_after(Duration::from_millis(16));
            }
        }

        let viewport = egui::ViewportBuilder::default()
            .with_title("Onevox Indicator")
            .with_decorations(false)
            .with_resizable(false)
            .with_transparent(false)
            .with_active(false)
            .with_always_on_top()
            .with_mouse_passthrough(true)
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]);

        #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
        let mut native_options = eframe::NativeOptions {
            viewport,
            ..Default::default()
        };

        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
            native_options.event_loop_builder = Some(Box::new(|builder| {
                builder.with_activation_policy(ActivationPolicy::Accessory);
            }));
        }

        eframe::run_native(
            "onevox-indicator",
            native_options,
            Box::new(move |cc| {
                let mut style = (*cc.egui_ctx.style()).clone();
                style.visuals.window_stroke = egui::Stroke::NONE;
                style.visuals.window_fill = egui::Color32::BLACK;
                style.visuals.panel_fill = egui::Color32::BLACK;
                style.visuals.window_shadow = egui::epaint::Shadow::NONE;
                style.visuals.popup_shadow = egui::epaint::Shadow::NONE;
                style.spacing.window_margin = egui::Margin::ZERO;
                cc.egui_ctx.set_style(style);
                cc.egui_ctx
                    .send_viewport_cmd(egui::ViewportCommand::MousePassthrough(true));

                Ok(Box::new(OverlayApp {
                    mode,
                    phase_start: Instant::now(),
                    positioned: false,
                    last_state_poll: Instant::now(),
                    frozen_phase: 0.0,
                }))
            }),
        )
        .map_err(|e| crate::Error::Other(format!("Indicator UI failed: {}", e)))?;

        Ok(())
    }
}
