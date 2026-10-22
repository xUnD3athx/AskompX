/*
 * Askomp
 *
 * DSP: UnD3ath, Plugin: UnD3ath
 *
 * Copyright (C) 2026 UnD3ath / Aqua Sounds
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

//! The Vizia editor, a close port of the original WPF user interface.

use atomic_float::AtomicF32;
use nih_plug::prelude::{Editor, Param, ParamPtr};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;
use nih_plug_vizia::widgets::RawParamEvent;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::Ordering;
use std::cell::Cell;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::{AskompParams, AskompShared, WAVE_SIZE};

/// The blue accent color used throughout the original UI (`#328cfa`).
const ACCENT: vg::Color = vg::Color::rgbaf(0.196, 0.549, 0.980, 1.0);
/// The accent color for Vizia's styled widgets.
/// The dark background of the waveform display (`#4c4c4c`).
/// The threshold range, in decibels.
const THRESHOLD_RANGE_DB: f32 = 36.0;

/// The decorative display font used for the title.
const TITLE_FONT: &str = "Odalisque NF";

/// The output meters show -60 dBFS at the left edge and 0 dBFS at the right.
const METER_FLOOR_DB: f32 = -60.0;
/// `METER_FLOOR_DB` as a linear amplitude.
const METER_FLOOR_LINEAR: f32 = 0.001;

/// Maps a linear peak amplitude to a `0..=1` meter fraction on a dB scale.
fn meter_fraction(linear: f32) -> f32 {
    if !linear.is_finite() || linear <= METER_FLOOR_LINEAR {
        return 0.0;
    }

    let db = 20.0 * linear.log10();
    ((db - METER_FLOOR_DB) / -METER_FLOOR_DB).clamp(0.0, 1.0)
}

/// The gain reduction meters show 0 dB at the center line and this level at
/// the outer edge.
const GR_METER_MAX_DB: f32 = 24.0;

/// Maps a gain reduction in dB to a `0..=1` bar fraction.
fn gr_fraction(db: f32) -> f32 {
    if !db.is_finite() || db <= 0.0 {
        return 0.0;
    }

    (db / GR_METER_MAX_DB).clamp(0.0, 1.0)
}

#[derive(Lens)]
struct Data {
    params: Arc<AskompParams>,

    // Display values, refreshed from the shared state on UI events.
    /// The current positive threshold, for the value label.
    pos_th_db: f32,
    /// The current negative threshold, for the value label.
    neg_th_db: f32,
    /// The current attack time, for the knob's value label.
    attack_ms: f32,
    /// The current release time, for the knob's value label.
    release_ms: f32,
    /// The current ratio, for the knob's value label.
    ratio: f32,
    /// The current knee width, for the knob's value label.
    knee: f32,
    /// The current stereo link amount in percent, for the knob's value label.
    stereo_link: f32,
    /// The current sidechain high-pass cutoff, for the knob's value label.
    hp_hz: f32,
    /// Whether the positive threshold strip is hovered, for the fading value
    /// label.
    pos_th_hover: bool,
    /// Whether the negative threshold strip is hovered, for the fading value
    /// label.
    neg_th_hover: bool,

    /// The hover flags written by the waveform view on every frame.
    positive_hovered: Arc<AtomicBool>,
    negative_hovered: Arc<AtomicBool>,
}

impl Data {
    /// Creates the data model.
    ///
    /// NOTE: There is intentionally **no background thread** driving these
    /// values. The values are refreshed whenever an event reaches this model:
    /// every parameter change (including the continuous stream emitted while
    /// dragging a knob or threshold line) and the waveform view's mouse events
    /// trigger an update. A `std::thread`-based update loop would outlive the
    /// editor and crash hosts that unload the plugin DLL right after deleting
    /// an instance (FL Studio).
    fn build(
        cx: &mut Context,
        params: Arc<AskompParams>,
        positive_hovered: Arc<AtomicBool>,
        negative_hovered: Arc<AtomicBool>,
    ) {
        let initial_attack = params.attack.value();
        let initial_release = params.release.value();
        let initial_ratio = params.ratio.value();
        let initial_knee = params.knee.value();
        let initial_stereo_link = params.stereo_link.value();
        let initial_hp = params.sidechain_hp.value();

        Data {
            params,
            pos_th_db: 0.0,
            neg_th_db: 0.0,
            attack_ms: initial_attack,
            release_ms: initial_release,
            ratio: initial_ratio,
            knee: initial_knee,
            stereo_link: initial_stereo_link,
            hp_hz: initial_hp,
            pos_th_hover: false,
            neg_th_hover: false,
            positive_hovered,
            negative_hovered,
        }
        .build(cx);
    }

    /// Refreshes all display values from the shared state and the parameters.
    fn sync_values(&mut self) {
        // The smoothing already happens per audio block on the audio thread
        // (matching the original's per-block UI updates), so the display
        // values are read directly.
        // The gain reduction numbers are drawn by `GrReadout`, which reads the
        // atomics every frame instead of caching them here.
        self.pos_th_db = self.params.positive_threshold.value();
        self.neg_th_db = self.params.negative_threshold.value();
        self.attack_ms = self.params.attack.value();
        self.release_ms = self.params.release.value();
        self.ratio = self.params.ratio.value();
        self.knee = self.params.knee.value();
        self.stereo_link = self.params.stereo_link.value();
        self.hp_hz = self.params.sidechain_hp.value();

        self.pos_th_hover = self.positive_hovered.load(Ordering::Relaxed);
        self.neg_th_hover = self.negative_hovered.load(Ordering::Relaxed);
    }
}

/// An update of the display values, emitted by the waveform view whenever the
/// mouse interacts with it.
#[derive(Debug, Clone, Copy)]
enum EditorEvent {
    UpdateMeters,
}

impl Model for Data {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        // Parameter changes stream through here while the user drags a knob
        // or a threshold line, which keeps the value labels in sync.
        event.map(|_param_event: &RawParamEvent, _| {
            self.sync_values();
        });

        event.map(|editor_event, meta| match *editor_event {
            EditorEvent::UpdateMeters => {
                self.sync_values();

                meta.consume();
            }
        });
    }
}

/// A horizontal output meter from the header. The left pill is rounded on its
/// left side and the right pill on its right side (`CornerRadius "10 0 0 10"`
/// and `"0 10 10 0"` in the original), and the blue fill is anchored on the
/// title's side of the pill, growing outwards.
struct PillMeter {
    shared: Arc<AskompShared>,
    kind: MeterKind,
    is_left: bool,
}

#[derive(Debug, Clone, Copy)]
enum MeterKind {
    OutputPeakLeft,
    OutputPeakRight,
}

impl PillMeter {
    pub fn new(
        cx: &mut Context,
        shared: Arc<AskompShared>,
        kind: MeterKind,
        is_left: bool,
    ) -> Handle<'_, Self> {
        Self {
            shared,
            kind,
            is_left,
        }
        .build(cx, |_| {})
    }

    fn level(&self) -> f32 {
        match self.kind {
            MeterKind::OutputPeakLeft => {
                self.shared.output_peak_left.load(Ordering::Relaxed)
            }
            MeterKind::OutputPeakRight => {
                self.shared.output_peak_right.load(Ordering::Relaxed)
            }
        }
    }
}

impl View for PillMeter {
    fn element(&self) -> Option<&'static str> {
        Some("pill-meter")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let radius = bounds.h / 2.0;
        let (outer_left, outer_right) = if self.is_left { (radius, 0.0) } else { (0.0, radius) };

        // The pill's background: `#33000000` on the white card.
        let mut path = vg::Path::new();
        path.rounded_rect_varying(
            bounds.x,
            bounds.y,
            bounds.w,
            bounds.h,
            outer_left,
            outer_right,
            outer_right,
            outer_left,
        );
        let paint = vg::Paint::color(vg::Color::rgbaf(0.0, 0.0, 0.0, 0.2));
        canvas.fill_path(&path, &paint);

        // The fill: inset by 5 (`Margin="5"`), anchored on the title's side.
        let level = meter_fraction(self.level());
        let inner_x = bounds.x + 5.0;
        let inner_w = bounds.w - 10.0;
        let inner_y = bounds.y + 5.0;
        let inner_h = bounds.h - 10.0;
        let fill_width = inner_w * level;
        let fill_x = if self.is_left {
            // The left meter is right-aligned, hugging the title.
            inner_x + inner_w - fill_width
        } else {
            inner_x
        };

        let (fill_left, fill_right) = if self.is_left { (5.0, 0.0) } else { (0.0, 5.0) };
        let mut fill = vg::Path::new();
        fill.rounded_rect_varying(
            fill_x,
            inner_y,
            fill_width,
            inner_h,
            fill_left,
            fill_right,
            fill_right,
            fill_left,
        );
        let paint = vg::Paint::color(ACCENT);
        canvas.fill_path(&fill, &paint);
    }
}

/// The vertical gain reduction meter on the left of the waveform display. The
/// positive reduction grows up from the center line and the negative reduction
/// grows down from it.
struct DualGrMeter {
    shared: Arc<AskompShared>,
}

impl DualGrMeter {
    pub fn new(cx: &mut Context, shared: Arc<AskompShared>) -> Handle<'_, Self> {
        Self { shared }.build(cx, |_| {})
    }
}

impl View for DualGrMeter {
    fn element(&self) -> Option<&'static str> {
        Some("dual-gr-meter")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        // The outer `#666` border with its rounded corners, and the inner
        // `#66000000` container inset by 5, exactly like the original's
        // nested borders. The inner radius is reduced by the inset so the
        // corners stay concentric.
        let outer_radius = bounds.w / 2.0;
        let mut outer = vg::Path::new();
        outer.rounded_rect(bounds.x, bounds.y, bounds.w, bounds.h, outer_radius);
        let paint = vg::Paint::color(vg::Color::rgbaf(0.4, 0.4, 0.4, 1.0));
        canvas.fill_path(&outer, &paint);

        let inner_x = bounds.x + 5.0;
        let inner_y = bounds.y + 5.0;
        let inner_w = bounds.w - 10.0;
        let inner_h = bounds.h - 10.0;
        let mut inner = vg::Path::new();
        inner.rounded_rect(inner_x, inner_y, inner_w, inner_h, outer_radius - 5.0);
        let paint = vg::Paint::color(vg::Color::rgbaf(0.0, 0.0, 0.0, 0.4));
        canvas.fill_path(&inner, &paint);

        // The two gain reduction fills, growing away from the center line.
        // The original uses `Margin="2"` and rounds the outer corners of each
        // half so they follow the container's curve.
        let center_y = bounds.y + bounds.h / 2.0;
        let fill_x = inner_x + 2.0;
        let fill_w = inner_w - 4.0;
        let fill_radius = outer_radius - 7.0;
        let avail = inner_h / 2.0 - 2.0;
        let positive_level =
            gr_fraction(self.shared.positive_compression_db.load(Ordering::Relaxed));
        let negative_level =
            gr_fraction(self.shared.negative_compression_db.load(Ordering::Relaxed));

        let mut fill = vg::Path::new();
        if positive_level > 0.0 {
            let fill_height = avail * positive_level;
            fill.rounded_rect_varying(
                fill_x,
                center_y - fill_height,
                fill_w,
                fill_height,
                fill_radius,
                fill_radius,
                0.0,
                0.0,
            );
        }
        if negative_level > 0.0 {
            let fill_height = avail * negative_level;
            fill.rounded_rect_varying(
                fill_x,
                center_y,
                fill_w,
                fill_height,
                0.0,
                0.0,
                fill_radius,
                fill_radius,
            );
        }
        let paint = vg::Paint::color(ACCENT);
        canvas.fill_path(&fill, &paint);

        // The center line, `#666` like the original.
        let mut line = vg::Path::new();
        line.move_to(fill_x, center_y);
        line.line_to(fill_x + fill_w, center_y);
        let mut paint = vg::Paint::color(vg::Color::rgbaf(0.4, 0.4, 0.4, 1.0));
        paint.set_line_width(1.0);
        canvas.stroke_path(&line, &paint);
    }
}

/// A numeric gain reduction readout drawn inside the GR meter. Unlike the
/// other value labels this is not bound to the `Data` model: the numbers have
/// to follow the audio every frame, and `Data` only refreshes on parameter and
/// mouse events. Reading the atomics while drawing keeps them live without a
/// background thread.
struct GrReadout {
    shared: Arc<AskompShared>,
    /// Whether this reads the negative half-wave's reduction.
    negative: bool,
    /// The font is registered with the canvas lazily, on the first draw.
    font: Cell<Option<vg::FontId>>,
}

impl GrReadout {
    fn new(cx: &mut Context, shared: Arc<AskompShared>, negative: bool) -> Handle<'_, Self> {
        Self {
            shared,
            negative,
            font: Cell::new(None),
        }
        .build(cx, |_| {})
    }

    fn value_db(&self) -> f32 {
        if self.negative {
            self.shared.negative_compression_db.load(Ordering::Relaxed)
        } else {
            self.shared.positive_compression_db.load(Ordering::Relaxed)
        }
    }

    fn text(&self) -> String {
        format!("{:.1}", self.value_db())
    }
}

impl View for GrReadout {
    fn element(&self) -> Option<&'static str> {
        Some("gr-readout")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let font = match self.font.get() {
            Some(font) => font,
            None => {
                let Ok(font) = canvas.add_font_mem(assets::fonts::NOTO_SANS_REGULAR) else {
                    return;
                };
                self.font.set(Some(font));
                font
            }
        };

        let mut paint = vg::Paint::color(vg::Color::rgbaf(1.0, 1.0, 1.0, 0.784));
        paint.set_font(&[font]);
        paint.set_font_size(10.0);
        paint.set_text_align(vg::Align::Center);
        paint.set_text_baseline(vg::Baseline::Middle);
        let _ = canvas.fill_text(
            bounds.x + bounds.w / 2.0,
            bounds.y + bounds.h / 2.0,
            self.text(),
            &paint,
        );
    }
}

/// Which of the two threshold lines is being dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragTarget {
    Positive,
    Negative,
}

/// The waveform display. Shows the stereo-mixed input and output waveforms
/// above and below the center line, and lets you drag the two threshold lines
/// around just like in the original WPF editor.
///
/// The bounds of this view correspond to the original's outer `#666` border;
/// the inner `#66000000` box is inset by 5. The threshold lines travel within
/// the inner box, but stop 10 pixels short of the center line, matching the
/// 10px tall drag strips of the original.
struct WaveformView {
    shared: Arc<AskompShared>,

    /// Whether each threshold strip is being hovered, shared with the threshold
    /// value labels so they can fade in and out.
    positive_hovered: Arc<AtomicBool>,
    negative_hovered: Arc<AtomicBool>,

    positive_ptr: ParamPtr,
    negative_ptr: ParamPtr,
    link_ptr: ParamPtr,

    /// The threshold line currently being dragged, if any.
    drag_target: Option<DragTarget>,
    /// The offset between the cursor and the dragged threshold line when the
    /// drag started, so the line follows the cursor relative to where the
    /// drag began instead of jumping to it.
    drag_offset: f32,
    /// Whether the link parameter was enabled when the drag started, so we
    /// know which parameters the gesture needs to be ended on.
    drag_started_linked: bool,
}

/// The width of the draggable threshold strip, from the original's
/// `Height="10"` borders.
const THRESHOLD_STRIP_HEIGHT: f32 = 10.0;

impl WaveformView {
    pub fn new(
        cx: &mut Context,
        params: Arc<AskompParams>,
        shared: Arc<AskompShared>,
        positive_hovered: Arc<AtomicBool>,
        negative_hovered: Arc<AtomicBool>,
    ) -> Handle<'_, Self> {
        Self {
            positive_ptr: params.positive_threshold.as_ptr(),
            negative_ptr: params.negative_threshold.as_ptr(),
            link_ptr: params.link.as_ptr(),
            shared,
            positive_hovered,
            negative_hovered,
            drag_target: None,
            drag_offset: 0.0,
            drag_started_linked: false,
        }
        .build(cx, |_| {})
    }

    /// The distance the threshold lines can travel from the outer edge of the
    /// inner box towards the center line.
    fn threshold_travel(height: f32) -> f32 {
        (height - 10.0) / 2.0 - THRESHOLD_STRIP_HEIGHT
    }

    /// The y-coordinate of a threshold line within this view, for the given
    /// view height. The positive line lives in the top half and the negative
    /// line in the bottom half.
    fn threshold_line_y(&self, height: f32, target: DragTarget) -> f32 {
        let threshold_db = match target {
            DragTarget::Positive => unsafe { self.positive_ptr.unmodulated_plain_value() },
            DragTarget::Negative => unsafe { self.negative_ptr.unmodulated_plain_value() },
        };

        let travel = Self::threshold_travel(height);
        let line_offset = travel * (-threshold_db / THRESHOLD_RANGE_DB);

        match target {
            DragTarget::Positive => 5.0 + line_offset,
            DragTarget::Negative => height - 5.0 - line_offset,
        }
    }

    /// Emits a `BeginSetParameter` event for the parameters involved in a
    /// drag gesture.
    fn begin_drag(&mut self, cx: &mut EventContext) {
        self.drag_started_linked = unsafe { self.link_ptr.unmodulated_plain_value() } > 0.5;
        cx.emit(RawParamEvent::BeginSetParameter(self.positive_ptr));
        if self.drag_started_linked {
            cx.emit(RawParamEvent::BeginSetParameter(self.negative_ptr));
        }
    }

    /// Ends the automation gesture started by `begin_drag()`.
    fn end_drag(&self, cx: &mut EventContext) {
        cx.emit(RawParamEvent::EndSetParameter(self.positive_ptr));
        if self.drag_started_linked {
            cx.emit(RawParamEvent::EndSetParameter(self.negative_ptr));
        }
    }

    /// Computes a new threshold value from a proposed threshold line
    /// y-coordinate within this view and updates the parameter(s).
    fn apply_drag(&self, cx: &mut EventContext, target: DragTarget, line_y: f32, height: f32) {
        let travel = Self::threshold_travel(height);

        // 0 dB sits at the outer edge of the waveform display, -36 dB at the
        // center line.
        let distance_from_edge = match target {
            DragTarget::Positive => line_y - 5.0,
            DragTarget::Negative => height - 5.0 - line_y,
        };
        let t = (distance_from_edge / travel).clamp(0.0, 1.0);
        let threshold_db = -THRESHOLD_RANGE_DB * t;

        let normalized = ((threshold_db + THRESHOLD_RANGE_DB) / THRESHOLD_RANGE_DB).clamp(0.0, 1.0);

        let linked = unsafe { self.link_ptr.unmodulated_plain_value() } > 0.5;
        let mut set = |ptr: ParamPtr, normalized: f32| {
            // Snap to the parameter's step size, like the built-in widgets do.
            let plain = unsafe { ptr.preview_plain(normalized) };
            let normalized = unsafe { ptr.preview_normalized(plain) };
            cx.emit(RawParamEvent::SetParameterNormalized(ptr, normalized));
        };

        match target {
            DragTarget::Positive => {
                set(self.positive_ptr, normalized);
                if linked {
                    set(self.negative_ptr, normalized);
                }
            }
            DragTarget::Negative => {
                set(self.negative_ptr, normalized);
                if linked {
                    set(self.positive_ptr, normalized);
                }
            }
        }
    }

    /// Draws one half of the waveform (either the positive or the negative
    /// side) as a polygon with the given paint.
    #[allow(clippy::too_many_arguments)]
    fn draw_waveform_half(
        canvas: &mut Canvas,
        left: f32,
        right: f32,
        center_y: f32,
        half: f32,
        positive: bool,
        data: &[AtomicF32],
        fill: bool,
        color: vg::Color,
    ) {
        let width = right - left;
        let mut path = vg::Path::new();
        // The baseline for both halves is the center line.
        path.move_to(left, center_y);
        for (i, sample) in data.iter().enumerate() {
            let x = left + (width * (i as f32 + 0.5) / WAVE_SIZE as f32);
            let value = sample.load(Ordering::Relaxed);
            let amplitude = if positive {
                value.max(0.0)
            } else {
                (-value).max(0.0)
            };
            let y = if positive {
                center_y - (amplitude * half)
            } else {
                center_y + (amplitude * half)
            };
            path.line_to(x, y);
        }
        path.line_to(right, center_y);

        let mut paint = vg::Paint::color(color);
        paint.set_line_width(1.2);
        if fill {
            canvas.fill_path(&path, &paint);
        }
        canvas.stroke_path(&path, &paint);
    }
}

impl View for WaveformView {
    fn element(&self) -> Option<&'static str> {
        Some("waveform")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) => {
                let bounds = cx.bounds();
                let y_local = cx.mouse().cursory - bounds.y;
                let positive_y = self.threshold_line_y(bounds.h, DragTarget::Positive);
                let negative_y = self.threshold_line_y(bounds.h, DragTarget::Negative);

                let target = if (y_local - positive_y).abs() <= (y_local - negative_y).abs() {
                    DragTarget::Positive
                } else {
                    DragTarget::Negative
                };

                // Relative drag: grab the line where it is instead of jumping
                // to the cursor position.
                let line_y = self.threshold_line_y(bounds.h, target);
                self.drag_target = Some(target);
                self.drag_offset = y_local - line_y;
                cx.capture();
                self.begin_drag(cx);

                meta.consume();
            }
            WindowEvent::MouseMove(_x, y) => {
                // The display values are refreshed on mouse activity; there is
                // deliberately no background thread driving them (see `Data`).
                cx.emit(EditorEvent::UpdateMeters);

                if let Some(target) = self.drag_target {
                    let bounds = cx.bounds();
                    let line_y = *y - bounds.y - self.drag_offset;
                    self.apply_drag(cx, target, line_y, bounds.h);

                    meta.consume();
                }
            }
            WindowEvent::MouseOver | WindowEvent::MouseOut => {
                // The hover flags are computed while drawing, so a refresh is
                // needed after the cursor enters or leaves the view.
                cx.emit(EditorEvent::UpdateMeters);
            }
            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.drag_target.take().is_some() {
                    cx.release();
                    self.end_drag(cx);

                    meta.consume();
                }
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        // The outer `#666` border and the inner `#66000000` box, matching the
        // original's nested borders.
        let mut outer = vg::Path::new();
        outer.rounded_rect(bounds.x, bounds.y, bounds.w, bounds.h, 10.0);
        let paint = vg::Paint::color(vg::Color::rgbaf(0.4, 0.4, 0.4, 1.0));
        canvas.fill_path(&outer, &paint);

        let inner_x = bounds.x + 5.0;
        let inner_y = bounds.y + 5.0;
        let inner_w = bounds.w - 10.0;
        let inner_h = bounds.h - 10.0;
        let mut inner = vg::Path::new();
        inner.rounded_rect(inner_x, inner_y, inner_w, inner_h, 5.0);
        let paint = vg::Paint::color(vg::Color::rgbaf(0.0, 0.0, 0.0, 0.4));
        canvas.fill_path(&inner, &paint);

        let center_y = bounds.y + bounds.h / 2.0;
        let half = inner_h / 2.0;

        // The waveforms span the whole inner box, like the original's canvases.
        let left = inner_x;
        let right = inner_x + inner_w;

        // Input waveforms, stroked in the accent color.
        Self::draw_waveform_half(canvas, left, right, center_y, half, true, &self.shared.wave_in, false, ACCENT);
        Self::draw_waveform_half(
            canvas, left, right, center_y, half, false, &self.shared.wave_in, false, ACCENT,
        );

        // Output waveforms, filled in white.
        Self::draw_waveform_half(
            canvas,
            left,
            right,
            center_y,
            half,
            true,
            &self.shared.wave_out,
            true,
            vg::Color::rgbaf(1.0, 1.0, 1.0, 0.5),
        );
        Self::draw_waveform_half(
            canvas,
            left,
            right,
            center_y,
            half,
            false,
            &self.shared.wave_out,
            true,
            vg::Color::rgbaf(1.0, 1.0, 1.0, 0.5),
        );

        // The center line, `#666` like the original.
        let mut line = vg::Path::new();
        line.move_to(left, center_y);
        line.line_to(right, center_y);
        let mut paint = vg::Paint::color(vg::Color::rgbaf(0.4, 0.4, 0.4, 1.0));
        paint.set_line_width(1.0);
        canvas.stroke_path(&line, &paint);

        // Threshold lines. Hovering a strip fades the line in from 0.2 to
        // full opacity, and it stays bright while being dragged.
        let mouse = cx.mouse();
        let cursor_x = mouse.cursorx;
        let cursor_y = mouse.cursory;
        let hovered = |line_y: f32| {
            cursor_x >= inner_x
                && cursor_x <= inner_x + inner_w
                && (cursor_y - line_y).abs() <= THRESHOLD_STRIP_HEIGHT / 2.0
        };

        for (target, hovered_flag) in [
            (DragTarget::Positive, &self.positive_hovered),
            (DragTarget::Negative, &self.negative_hovered),
        ] {
            let line_y = bounds.y + self.threshold_line_y(bounds.h, target);
            let is_dragged = self.drag_target == Some(target);
            let is_hovered = !is_dragged && hovered(line_y);
            hovered_flag.store(is_hovered || is_dragged, Ordering::Relaxed);

            let mut line = vg::Path::new();
            line.move_to(left, line_y);
            line.line_to(right, line_y);

            let alpha = if is_dragged || is_hovered { 1.0 } else { 0.2 };
            let mut paint = vg::Paint::color(vg::Color::rgbaf(1.0, 1.0, 1.0, alpha));
            paint.set_line_width(1.0);
            canvas.stroke_path(&line, &paint);
        }
    }
}

/// Which parameter a knob controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KnobKind {
    Attack,
    Release,
    Ratio,
    Knee,
    StereoLink,
    SidechainHP,
}

/// A rotary knob, ported from the original's ellipse-based knobs. Drag
/// vertically to change the value, double click to reset it to the default.
struct KnobView {
    kind: KnobKind,
    ptr: ParamPtr,
    min: f32,
    max: f32,
    /// How many plain value units one pixel of dragging adds.
    drag_units_per_pixel: f32,
    /// The stroke width of the value arc. The original uses a thicker arc for
    /// the large ratio knob.
    arc_width: f32,

    /// The current hover scale animation state, from 0 to 1.
    hover_scale: Cell<f32>,

    dragging: bool,
    drag_start_y: f32,
    drag_start_value: f32,
}

impl KnobView {
    pub fn new<'a>(
        cx: &'a mut Context,
        params: &'a Arc<AskompParams>,
        kind: KnobKind,
    ) -> Handle<'a, Self> {
        let (ptr, min, max, drag_units_per_pixel, arc_width) = match kind {
            KnobKind::Attack => (
                params.attack.as_ptr(),
                0.0,
                50.0,
                0.1,
                3.0,
            ),
            KnobKind::Release => (
                params.release.as_ptr(),
                20.0,
                200.0,
                0.5,
                3.0,
            ),
            KnobKind::Ratio => (
                params.ratio.as_ptr(),
                1.0,
                20.0,
                0.1,
                5.0,
            ),
            KnobKind::Knee => (
                params.knee.as_ptr(),
                0.0,
                24.0,
                0.1,
                3.0,
            ),
            KnobKind::StereoLink => (
                params.stereo_link.as_ptr(),
                0.0,
                100.0,
                0.5,
                3.0,
            ),
            KnobKind::SidechainHP => (
                params.sidechain_hp.as_ptr(),
                20.0,
                500.0,
                2.0,
                3.0,
            ),
        };

        Self {
            kind,
            ptr,
            min,
            max,
            drag_units_per_pixel,
            arc_width,
            hover_scale: Cell::new(0.0),
            dragging: false,
            drag_start_y: 0.0,
            drag_start_value: 0.0,
        }
        .build(cx, |_| {})
    }

    /// Emits a set-parameter event for a plain value, snapped to the
    /// parameter's step size.
    fn set_plain_value(&self, cx: &mut EventContext, plain_value: f32) {
        let plain_value = plain_value.clamp(self.min, self.max);
        let normalized = unsafe { self.ptr.preview_normalized(plain_value) };
        let snapped = unsafe { self.ptr.preview_plain(normalized) };
        let normalized = unsafe { self.ptr.preview_normalized(snapped) };
        cx.emit(RawParamEvent::SetParameterNormalized(self.ptr, normalized));
    }
}

impl View for KnobView {
    fn element(&self) -> Option<&'static str> {
        Some("knob")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) => {
                self.dragging = true;
                self.drag_start_y = cx.mouse().cursory;
                self.drag_start_value = unsafe { self.ptr.unmodulated_plain_value() };
                cx.capture();
                cx.emit(RawParamEvent::BeginSetParameter(self.ptr));

                meta.consume();
            }
            WindowEvent::MouseDoubleClick(MouseButton::Left) => {
                // Reset to the default value, like the original's double click
                // behavior. The gesture from the initial click is ended first.
                if self.dragging {
                    self.dragging = false;
                    cx.release();
                    cx.emit(RawParamEvent::EndSetParameter(self.ptr));
                }

                cx.emit(RawParamEvent::BeginSetParameter(self.ptr));
                let default_value = unsafe { self.ptr.default_plain_value() };
                self.set_plain_value(cx, default_value);
                cx.emit(RawParamEvent::EndSetParameter(self.ptr));

                meta.consume();
            }
            WindowEvent::MouseMove(_x, y) => {
                if self.dragging {
                    let delta = (self.drag_start_y - *y) * self.drag_units_per_pixel;
                    self.set_plain_value(cx, self.drag_start_value + delta);

                    meta.consume();
                }
            }
            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.dragging {
                    self.dragging = false;
                    cx.release();
                    cx.emit(RawParamEvent::EndSetParameter(self.ptr));

                    meta.consume();
                }
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let center_x = bounds.x + bounds.w / 2.0;
        let center_y = bounds.y + bounds.h / 2.0;

        let large = self.kind == KnobKind::Ratio;

        // Hovering a knob scales it up by 2 pixels with a short ease, like the
        // original's `MinWidth + 2` hover animation. The animation runs for
        // free because the view is redrawn every frame.
        let mouse = cx.mouse();
        let hovered = mouse.cursorx >= bounds.x
            && mouse.cursorx <= bounds.x + bounds.w
            && mouse.cursory >= bounds.y
            && mouse.cursory <= bounds.y + bounds.h;
        let target_scale = if hovered { 1.0 } else { 0.0 };
        let scale = self.hover_scale.get() + (target_scale - self.hover_scale.get()) * 0.3;
        self.hover_scale.set(scale);

        let radius = bounds.w.min(bounds.h) / 2.0 - 1.0 + scale;
        let arc_width = self.arc_width;

        // The outer white circle with its gray border.
        let mut circle = vg::Path::new();
        circle.arc(center_x, center_y, radius, 0.0, std::f32::consts::TAU, vg::Solidity::Solid);
        circle.close();
        let paint = vg::Paint::color(vg::Color::rgbaf(1.0, 1.0, 1.0, 1.0));
        canvas.fill_path(&circle, &paint);
        let mut paint = vg::Paint::color(vg::Color::rgbaf(0.4, 0.4, 0.4, 1.0));
        paint.set_line_width(4.0);
        canvas.stroke_path(&circle, &paint);

        // The value arc, drawn at the outer edge of the knob and starting at
        // the bottom, sweeping clockwise, just like the original.
        let normalized = unsafe {
            let plain = self.ptr.unmodulated_plain_value();
            (plain - self.min) / (self.max - self.min)
        }
        .clamp(0.0, 1.0);
        if normalized > 0.0 {
            let arc_radius = radius - arc_width / 2.0;
            let mut arc = vg::Path::new();
            arc.arc(
                center_x,
                center_y,
                arc_radius,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::FRAC_PI_2 + normalized * std::f32::consts::TAU,
                vg::Solidity::Hole,
            );
            let mut paint = vg::Paint::color(ACCENT);
            paint.set_line_width(arc_width);
            canvas.stroke_path(&arc, &paint);
        }

        // The blue accent ring and the inner face of the knob. The original
        // stacks ellipses with `Margin="6"`/`"8"` on the small knobs and
        // `Margin="8"`/`"10"` on the ratio knob.
        let ring_radius = radius - if large { 8.0 } else { 6.0 };
        let mut ring = vg::Path::new();
        ring.arc(center_x, center_y, ring_radius, 0.0, std::f32::consts::TAU, vg::Solidity::Solid);
        ring.close();
        let paint = vg::Paint::color(ACCENT);
        canvas.fill_path(&ring, &paint);

        let inner_radius = radius - if large { 10.0 } else { 8.0 };
        let mut face = vg::Path::new();
        face.arc(center_x, center_y, inner_radius, 0.0, std::f32::consts::TAU, vg::Solidity::Solid);
        face.close();
        if large {
            // The large knob has a gray face with a dark border.
            let paint = vg::Paint::color(vg::Color::rgbaf(0.4, 0.4, 0.4, 1.0));
            canvas.fill_path(&face, &paint);
            let mut paint = vg::Paint::color(vg::Color::rgbaf(0.2, 0.2, 0.2, 1.0));
            paint.set_line_width(2.0);
            canvas.stroke_path(&face, &paint);
        } else {
            // The small knobs have a white face.
            let paint = vg::Paint::color(vg::Color::rgbaf(1.0, 1.0, 1.0, 1.0));
            canvas.fill_path(&face, &paint);
        }
    }
}

/// The pill-shaped `LINK` toggle below the waveform display.
struct LinkButton {
    link_ptr: ParamPtr,
}

impl LinkButton {
    pub fn new<'a>(cx: &'a mut Context, params: &'a Arc<AskompParams>) -> Handle<'a, Self> {
        Self {
            link_ptr: params.link.as_ptr(),
        }
        .build(cx, |cx| {
            Label::new(cx, "LINK")
                .font_size(8.0)
                .font_weight(FontWeightKeyword::Light)
                .color(Color::rgb(255, 255, 255))
                .top(Stretch(1.0))
                .bottom(Stretch(1.0))
                .left(Stretch(1.0))
                .right(Stretch(1.0));
        })
    }
}

impl View for LinkButton {
    fn element(&self) -> Option<&'static str> {
        Some("link-button")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) => {
                let linked = unsafe { self.link_ptr.unmodulated_plain_value() } > 0.5;
                cx.emit(RawParamEvent::BeginSetParameter(self.link_ptr));
                cx.emit(RawParamEvent::SetParameterNormalized(
                    self.link_ptr,
                    if linked { 0.0 } else { 1.0 },
                ));
                cx.emit(RawParamEvent::EndSetParameter(self.link_ptr));

                meta.consume();
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let mouse = cx.mouse();
        let hovered = mouse.cursorx >= bounds.x
            && mouse.cursorx <= bounds.x + bounds.w
            && mouse.cursory >= bounds.y
            && mouse.cursory <= bounds.y + bounds.h;

        // The original's four background colors: blue when linked, gray when
        // not, with darker hover variants for both states.
        let linked = unsafe { self.link_ptr.unmodulated_plain_value() } > 0.5;
        let color = match (linked, hovered) {
            (true, true) => vg::Color::rgbaf(0x28 as f32 / 255.0, 0x72 as f32 / 255.0, 0xcc as f32 / 255.0, 1.0),
            (true, false) => ACCENT,
            (false, true) => vg::Color::rgbaf(0x99 as f32 / 255.0, 0x99 as f32 / 255.0, 0x99 as f32 / 255.0, 1.0),
            (false, false) => vg::Color::rgbaf(0xcc as f32 / 255.0, 0xcc as f32 / 255.0, 0xcc as f32 / 255.0, 1.0),
        };

        let mut path = vg::Path::new();
        path.rounded_rect(bounds.x, bounds.y, bounds.w, bounds.h, bounds.h / 2.0);
        let paint = vg::Paint::color(color);
        canvas.fill_path(&path, &paint);
    }
}

/// The round `delta` toggle at the right end of the knob row. When enabled,
/// the plugin outputs the difference between the processed and the input
/// signal instead of the processed signal.
struct DeltaButton {
    delta_ptr: ParamPtr,
}

impl DeltaButton {
    pub fn new<'a>(cx: &'a mut Context, params: &'a Arc<AskompParams>) -> Handle<'a, Self> {
        Self {
            delta_ptr: params.delta.as_ptr(),
        }
        .build(cx, |cx| {
            Label::new(cx, "delta")
                .font_size(8.0)
                .font_weight(FontWeightKeyword::Light)
                .color(Color::rgb(255, 255, 255))
                .top(Stretch(1.0))
                .bottom(Stretch(1.0))
                .left(Stretch(1.0))
                .right(Stretch(1.0));
        })
    }
}

impl View for DeltaButton {
    fn element(&self) -> Option<&'static str> {
        Some("delta-button")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) => {
                let enabled = unsafe { self.delta_ptr.unmodulated_plain_value() } > 0.5;
                cx.emit(RawParamEvent::BeginSetParameter(self.delta_ptr));
                cx.emit(RawParamEvent::SetParameterNormalized(
                    self.delta_ptr,
                    if enabled { 0.0 } else { 1.0 },
                ));
                cx.emit(RawParamEvent::EndSetParameter(self.delta_ptr));

                meta.consume();
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let mouse = cx.mouse();
        let hovered = mouse.cursorx >= bounds.x
            && mouse.cursorx <= bounds.x + bounds.w
            && mouse.cursory >= bounds.y
            && mouse.cursory <= bounds.y + bounds.h;

        // Same four-state colors as the LINK toggle, but drawn as a circle.
        let enabled = unsafe { self.delta_ptr.unmodulated_plain_value() } > 0.5;
        let color = match (enabled, hovered) {
            (true, true) => vg::Color::rgbaf(0x28 as f32 / 255.0, 0x72 as f32 / 255.0, 0xcc as f32 / 255.0, 1.0),
            (true, false) => ACCENT,
            (false, true) => vg::Color::rgbaf(0x99 as f32 / 255.0, 0x99 as f32 / 255.0, 0x99 as f32 / 255.0, 1.0),
            (false, false) => vg::Color::rgbaf(0xcc as f32 / 255.0, 0xcc as f32 / 255.0, 0xcc as f32 / 255.0, 1.0),
        };

        let center_x = bounds.x + bounds.w / 2.0;
        let center_y = bounds.y + bounds.h / 2.0;
        let radius = bounds.w.min(bounds.h) / 2.0;
        let mut circle = vg::Path::new();
        circle.arc(
            center_x,
            center_y,
            radius,
            0.0,
            std::f32::consts::TAU,
            vg::Solidity::Solid,
        );
        circle.close();
        let paint = vg::Paint::color(color);
        canvas.fill_path(&circle, &paint);
    }
}

/// The waveform speed slider from the original's settings panel: 8 ticks that
/// set how many process blocks pass before the waveform display scrolls by one
/// column. Writes straight to the shared state and reads it back while
/// drawing, so it needs no model of its own.
struct WaveSpeedSlider {
    shared: Arc<AskompShared>,
    dragging: bool,
}

/// The number of discrete ticks on the speed slider.
const SPEED_TICKS: usize = 8;

impl WaveSpeedSlider {
    pub fn new(cx: &mut Context, shared: Arc<AskompShared>) -> Handle<'_, Self> {
        Self {
            shared,
            dragging: false,
        }
        .build(cx, |_| {})
    }

    /// The current tick, derived from the shared scroll step exactly like the
    /// original's `_step = 9 - value`.
    fn speed(&self) -> usize {
        9usize
            .saturating_sub(self.shared.wave_step.load(Ordering::Relaxed))
            .min(SPEED_TICKS - 1)
    }

    fn set_speed(&self, speed: usize) {
        let speed = speed.min(SPEED_TICKS - 1);
        self.shared
            .wave_step
            .store(9 - speed, Ordering::Relaxed);
    }
}

impl View for WaveSpeedSlider {
    fn element(&self) -> Option<&'static str> {
        Some("wave-speed-slider")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) | WindowEvent::MouseMove(..) => {
                let is_down = matches!(window_event, WindowEvent::MouseDown(MouseButton::Left));
                if !self.dragging && !is_down {
                    return;
                }

                if is_down {
                    self.dragging = true;
                    cx.capture();
                }

                let bounds = cx.bounds();
                let speed = (((cx.mouse().cursorx - bounds.x) / bounds.w * (SPEED_TICKS - 1) as f32)
                    .round() as usize)
                    .clamp(0, SPEED_TICKS - 1);
                self.set_speed(speed);

                meta.consume();
            }
            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.dragging {
                    self.dragging = false;
                    cx.release();

                    meta.consume();
                }
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        // The thin rounded track, like the original's `Height="5"` border.
        let track_y = bounds.y + bounds.h / 2.0 - 2.5;
        let mut track = vg::Path::new();
        track.rounded_rect(bounds.x, track_y, bounds.w, 5.0, 2.0);
        let paint = vg::Paint::color(vg::Color::rgbaf(0.0, 0.0, 0.0, 0.2));
        canvas.fill_path(&track, &paint);

        // The blue fill up to the current tick, and the white thumb at its
        // end.
        let fill_w = (bounds.w - 10.0) * (self.speed() as f32 / (SPEED_TICKS - 1) as f32);
        if fill_w > 0.0 {
            let mut fill = vg::Path::new();
            fill.rounded_rect(bounds.x + 5.0, track_y, fill_w, 5.0, 2.0);
            let paint = vg::Paint::color(ACCENT);
            canvas.fill_path(&fill, &paint);
        }

        let mut thumb = vg::Path::new();
        thumb.arc(
            bounds.x + 5.0 + fill_w,
            bounds.y + bounds.h / 2.0,
            5.0,
            0.0,
            std::f32::consts::TAU,
            vg::Solidity::Solid,
        );
        thumb.close();
        let paint = vg::Paint::color(vg::Color::rgbaf(1.0, 1.0, 1.0, 1.0));
        canvas.fill_path(&thumb, &paint);
        let mut paint = vg::Paint::color(vg::Color::rgbaf(0.4, 0.4, 0.4, 1.0));
        paint.set_line_width(1.0);
        canvas.stroke_path(&thumb, &paint);
    }
}

pub(crate) fn create(
    params: Arc<AskompParams>,
    shared: Arc<AskompShared>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);
        cx.add_font_mem(include_bytes!("../assets/OdalisqueNf-POwg.ttf").as_slice());

        let positive_hovered = Arc::new(AtomicBool::new(false));
        let negative_hovered = Arc::new(AtomicBool::new(false));

        Data::build(
            cx,
            params.clone(),
            positive_hovered.clone(),
            negative_hovered.clone(),
        );

        // A fixed 500x400 pixel layout, positioned absolutely to match the
        // original WPF editor's geometry element by element. Vizia's
        // stretch-based layout is not reliable across hosts with different
        // scale factors, so every element is placed at explicit coordinates.
        ZStack::new(cx, |cx| {
            // The little vendor label in the top left corner of the window.
            Label::new(cx, "Aqua Sounds")
                .font_size(10.0)
                .color(Color::rgba(255, 255, 255, 110))
                .left(Pixels(8.0))
                .top(Pixels(2.0))
                .width(Pixels(120.0))
                .height(Pixels(14.0))
                .pointer_events(false);

            // The white rounded card on the gray window background.
            Element::new(cx)
                .background_color(Color::rgb(255, 255, 255))
                .border_radius(Pixels(10.0))
                .left(Pixels(5.0))
                .top(Pixels(5.0))
                .width(Pixels(490.0))
                .height(Pixels(390.0));

            // The header: an output meter pill on either side of the title.
            // Both pills sit 20px inside the card, 20px tall, vertically
            // centered in the 72px tall header row.
            PillMeter::new(cx, shared.clone(), MeterKind::OutputPeakLeft, true)
                .left(Pixels(25.0))
                .top(Pixels(40.0))
                .width(Pixels(135.0))
                .height(Pixels(20.0));

            Label::new(cx, "AskompX")
                .font_family(vec![FamilyOwned::Name(String::from(TITLE_FONT))])
                .font_size(24.0)
                .color(Color::rgb(26, 26, 26))
                .left(Pixels(185.0))
                .top(Pixels(26.0))
                .width(Pixels(130.0))
                .height(Pixels(38.0))
                .text_align(TextAlign::Center)
                .pointer_events(false);

            PillMeter::new(cx, shared.clone(), MeterKind::OutputPeakRight, false)
                .left(Pixels(340.0))
                .top(Pixels(40.0))
                .width(Pixels(135.0))
                .height(Pixels(20.0));

            // The gain reduction meter, with the positive and negative
            // reduction shown as separate numeric values inside the top and
            // bottom of the bar like the original. Both readouts are created
            // after the meter so they render on top of it.
            DualGrMeter::new(cx, shared.clone())
                .left(Pixels(25.0))
                .top(Pixels(77.0))
                .width(Pixels(30.0))
                .height(Pixels(185.0));

            GrReadout::new(cx, shared.clone(), false)
                .left(Pixels(25.0))
                .top(Pixels(84.0))
                .width(Pixels(30.0))
                .height(Pixels(12.0))
                .pointer_events(false);

            GrReadout::new(cx, shared.clone(), true)
                .left(Pixels(25.0))
                .top(Pixels(239.0))
                .width(Pixels(30.0))
                .height(Pixels(12.0))
                .pointer_events(false);

            // The waveform display.
            WaveformView::new(
                cx,
                params.clone(),
                shared.clone(),
                positive_hovered.clone(),
                negative_hovered.clone(),
            )
            .left(Pixels(65.0))
            .top(Pixels(77.0))
            .width(Pixels(410.0))
            .height(Pixels(185.0));

            // The threshold value labels, centered in each half of the
            // waveform display and faded in while the corresponding threshold
            // strip is hovered.
            Label::new(cx, Data::pos_th_db.map(|th| format!("{th:.1} dB")))
                .font_size(20.0)
                .color(Color::rgba(255, 255, 255, 60))
                .opacity(Data::pos_th_hover.map(|hovered| if *hovered { 1.0 } else { 0.0 }))
                .left(Pixels(70.0))
                .top(Pixels(112.0))
                .width(Pixels(400.0))
                .height(Pixels(28.0))
                .text_align(TextAlign::Center)
                .pointer_events(false);

            Label::new(cx, Data::neg_th_db.map(|th| format!("{th:.1} dB")))
                .font_size(20.0)
                .color(Color::rgba(255, 255, 255, 60))
                .opacity(Data::neg_th_hover.map(|hovered| if *hovered { 1.0 } else { 0.0 }))
                .left(Pixels(70.0))
                .top(Pixels(201.0))
                .width(Pixels(400.0))
                .height(Pixels(28.0))
                .text_align(TextAlign::Center)
                .pointer_events(false);

            // The waveform speed slider, between the waveform display and the
            // LINK toggle.
            Label::new(cx, "Speed")
                .font_size(9.0)
                .color(Color::rgb(120, 120, 120))
                .left(Pixels(76.0))
                .top(Pixels(266.0))
                .width(Pixels(54.0))
                .height(Pixels(10.0))
                .pointer_events(false);

            WaveSpeedSlider::new(cx, shared.clone())
                .left(Pixels(134.0))
                .top(Pixels(265.0))
                .width(Pixels(336.0))
                .height(Pixels(12.0));

            // The LINK toggle, centered horizontally like the original.
            LinkButton::new(cx, &params)
                .left(Pixels(225.0))
                .top(Pixels(282.0))
                .width(Pixels(50.0))
                .height(Pixels(20.0));

            // The knob row: HP, stereo link, attack, the large ratio knob,
            // release and knee, all vertically centered at y = 345. Attack and
            // release sit symmetrically around the centered ratio knob, with
            // their value labels below the small knobs.
            ZStack::new(cx, |cx| {
                KnobView::new(cx, &params, KnobKind::Attack).size(Pixels(46.0));

                Label::new(cx, "A")
                    .font_size(12.0)
                    .color(Color::rgb(51, 51, 51))
                    .top(Stretch(1.0))
                    .bottom(Stretch(1.0))
                    .left(Stretch(1.0))
                    .right(Stretch(1.0))
                    .pointer_events(false);
            })
            .left(Pixels(147.0))
            .top(Pixels(322.0))
            .size(Pixels(46.0));

            Label::new(cx, Data::attack_ms.map(|v| format!("{v:.0}ms")))
                .font_size(12.0)
                .color(Color::rgb(51, 51, 51))
                .left(Pixels(147.0))
                .top(Pixels(372.0))
                .width(Pixels(46.0))
                .height(Pixels(16.0))
                .text_align(TextAlign::Center)
                .pointer_events(false);

            // The sidechain high-pass cutoff.
            ZStack::new(cx, |cx| {
                KnobView::new(cx, &params, KnobKind::SidechainHP).size(Pixels(36.0));

                Label::new(cx, "HP")
                    .font_size(10.0)
                    .color(Color::rgb(51, 51, 51))
                    .top(Stretch(1.0))
                    .bottom(Stretch(1.0))
                    .left(Stretch(1.0))
                    .right(Stretch(1.0))
                    .pointer_events(false);
            })
            .left(Pixels(35.0))
            .top(Pixels(327.0))
            .size(Pixels(36.0));

            Label::new(cx, Data::hp_hz.map(|v| format!("{v:.0}Hz")))
                .font_size(12.0)
                .color(Color::rgb(51, 51, 51))
                .left(Pixels(35.0))
                .top(Pixels(368.0))
                .width(Pixels(36.0))
                .height(Pixels(16.0))
                .text_align(TextAlign::Center)
                .pointer_events(false);

            ZStack::new(cx, |cx| {
                KnobView::new(cx, &params, KnobKind::Ratio).size(Pixels(80.0));

                Label::new(cx, Data::ratio.map(|v| format!("{v:.0} : 1")))
                    .font_size(18.0)
                    .color(Color::rgb(255, 255, 255))
                    .top(Stretch(1.0))
                    .bottom(Stretch(1.0))
                    .left(Stretch(1.0))
                    .right(Stretch(1.0))
                    .pointer_events(false);
            })
            .left(Pixels(210.0))
            .top(Pixels(307.0))
            .size(Pixels(80.0));

            ZStack::new(cx, |cx| {
                KnobView::new(cx, &params, KnobKind::StereoLink).size(Pixels(36.0));

                Label::new(cx, "S")
                    .font_size(12.0)
                    .color(Color::rgb(51, 51, 51))
                    .top(Stretch(1.0))
                    .bottom(Stretch(1.0))
                    .left(Stretch(1.0))
                    .right(Stretch(1.0))
                    .pointer_events(false);
            })
            .left(Pixels(91.0))
            .top(Pixels(327.0))
            .size(Pixels(36.0));

            Label::new(cx, Data::stereo_link.map(|v| format!("{v:.0}%")))
                .font_size(12.0)
                .color(Color::rgb(51, 51, 51))
                .left(Pixels(91.0))
                .top(Pixels(368.0))
                .width(Pixels(36.0))
                .height(Pixels(16.0))
                .text_align(TextAlign::Center)
                .pointer_events(false);

            ZStack::new(cx, |cx| {
                KnobView::new(cx, &params, KnobKind::Release).size(Pixels(50.0));

                Label::new(cx, "R")
                    .font_size(12.0)
                    .color(Color::rgb(51, 51, 51))
                    .top(Stretch(1.0))
                    .bottom(Stretch(1.0))
                    .left(Stretch(1.0))
                    .right(Stretch(1.0))
                    .pointer_events(false);
            })
            .left(Pixels(305.0))
            .top(Pixels(320.0))
            .size(Pixels(50.0));

            Label::new(cx, Data::release_ms.map(|v| format!("{v:.0}ms")))
                .font_size(12.0)
                .color(Color::rgb(51, 51, 51))
                .left(Pixels(305.0))
                .top(Pixels(374.0))
                .width(Pixels(50.0))
                .height(Pixels(16.0))
                .text_align(TextAlign::Center)
                .pointer_events(false);

            ZStack::new(cx, |cx| {
                KnobView::new(cx, &params, KnobKind::Knee).size(Pixels(36.0));

                Label::new(cx, "K")
                    .font_size(12.0)
                    .color(Color::rgb(51, 51, 51))
                    .top(Stretch(1.0))
                    .bottom(Stretch(1.0))
                    .left(Stretch(1.0))
                    .right(Stretch(1.0))
                    .pointer_events(false);
            })
            .left(Pixels(373.0))
            .top(Pixels(327.0))
            .size(Pixels(36.0));

            Label::new(cx, Data::knee.map(|v| format!("{v:.1}dB")))
                .font_size(12.0)
                .color(Color::rgb(51, 51, 51))
                .left(Pixels(373.0))
                .top(Pixels(368.0))
                .width(Pixels(36.0))
                .height(Pixels(16.0))
                .text_align(TextAlign::Center)
                .pointer_events(false);

            // The round delta toggle, mirroring the sidechain high-pass knob
            // on the left edge of the knob row.
            DeltaButton::new(cx, &params)
                .left(Pixels(429.0))
                .top(Pixels(327.0))
                .size(Pixels(36.0));
        })
        .background_color(Color::rgb(102, 102, 102));
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a readout over a freshly allocated shared state, returning both
    /// so tests can mutate the atomics while holding the readout.
    fn fresh_readout(negative: bool) -> (Arc<AskompShared>, GrReadout) {
        let shared = Arc::new(AskompShared::new(Arc::default()));
        let readout = GrReadout {
            shared: shared.clone(),
            negative,
            font: std::cell::Cell::new(None),
        };
        (shared, readout)
    }

    /// A freshly created shared state has never compressed anything, so the
    /// readout shows `0.0` rather than an empty string.
    #[test]
    fn fresh_shared_state_reads_as_zero() {
        let (_shared, readout) = fresh_readout(false);

        assert_eq!(readout.value_db(), 0.0);
        assert_eq!(readout.text(), "0.0");
    }

    /// A published positive gain reduction is formatted with one decimal.
    #[test]
    fn positive_value_is_formatted_with_one_decimal() {
        let (shared, readout) = fresh_readout(false);
        shared
            .positive_compression_db
            .store(6.04, std::sync::atomic::Ordering::Relaxed);

        assert_eq!(readout.text(), "6.0");
    }

    /// The value is read on every call instead of being cached at
    /// construction, which is what keeps the meter live during playback.
    #[test]
    fn text_reflects_later_atomic_updates() {
        let (shared, readout) = fresh_readout(false);

        shared
            .positive_compression_db
            .store(6.04, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(readout.text(), "6.0");

        shared
            .positive_compression_db
            .store(12.0, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(readout.text(), "12.0");
    }

    /// A negative-polarity readout follows the negative channel and ignores
    /// the positive one, even when both are published.
    #[test]
    fn negative_readout_uses_negative_channel() {
        let (shared, readout) = fresh_readout(true);
        shared
            .positive_compression_db
            .store(3.0, std::sync::atomic::Ordering::Relaxed);
        shared
            .negative_compression_db
            .store(9.0, std::sync::atomic::Ordering::Relaxed);

        assert_eq!(readout.value_db(), 9.0);
        assert_eq!(readout.text(), "9.0");
    }

    /// The text rounds to one decimal exactly like `format!("{:.1}")`.
    #[test]
    fn text_rounds_to_one_decimal() {
        let (shared, readout) = fresh_readout(false);

        shared
            .positive_compression_db
            .store(3.24, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(readout.text(), "3.2");

        shared
            .positive_compression_db
            .store(3.26, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(readout.text(), "3.3");
    }

    /// `value_db()` returns the raw stored value for both polarities, without
    /// any rounding applied.
    #[test]
    fn value_db_returns_unrounded_value_for_both_polarities() {
        let (shared, positive) = fresh_readout(false);
        let negative = GrReadout {
            shared: shared.clone(),
            negative: true,
            font: std::cell::Cell::new(None),
        };
        shared
            .positive_compression_db
            .store(2.5, std::sync::atomic::Ordering::Relaxed);
        shared
            .negative_compression_db
            .store(7.25, std::sync::atomic::Ordering::Relaxed);

        assert_eq!(positive.value_db(), 2.5);
        assert_eq!(negative.value_db(), 7.25);
    }

    /// The tolerance allowed for the dB-scale fractions.
    const FRACTION_EPSILON: f32 = 1e-3;

    /// Asserts that a fraction matches `expected` within `FRACTION_EPSILON`.
    /// Exact equality is reserved for the guaranteed `0.0` and `1.0` cases.
    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= FRACTION_EPSILON,
            "expected {expected} +/- {FRACTION_EPSILON}, got {actual}"
        );
    }

    /// Silence and negative amplitudes read as an empty output meter: the
    /// amplitude is rejected before any logarithm is taken.
    #[test]
    fn meter_fraction_zero_and_negative_amplitude_are_empty() {
        assert_eq!(meter_fraction(0.0), 0.0);
        assert_eq!(meter_fraction(-0.5), 0.0);
    }

    /// 0 dBFS lands on the right edge of the meter.
    #[test]
    fn meter_fraction_full_scale_fills_the_bar() {
        assert_eq!(meter_fraction(1.0), 1.0);
    }

    /// A half-amplitude peak is about -6.02 dBFS, roughly 90% of the dB scale
    /// instead of the 50% the old linear fill showed.
    #[test]
    fn meter_fraction_half_amplitude_is_about_ninety_percent() {
        assert_close(meter_fraction(0.5), 0.89966);
    }

    /// -40 dBFS is two thirds of the way from -60 dBFS up to 0 dBFS.
    #[test]
    fn meter_fraction_minus_forty_db_is_one_third() {
        assert_close(meter_fraction(0.01), 1.0 / 3.0);
    }

    /// The meter floor is exactly -60 dBFS, so a 0.001 amplitude is empty.
    #[test]
    fn meter_fraction_at_the_floor_is_zero() {
        assert_eq!(meter_fraction(0.001), 0.0);
    }

    /// Anything at or above 0 dBFS is clamped to a full bar.
    #[test]
    fn meter_fraction_above_full_scale_is_clamped() {
        assert_eq!(meter_fraction(2.0), 1.0);
    }

    /// A NaN peak must not poison the pill geometry, so it reads as empty.
    #[test]
    fn meter_fraction_nan_is_zero() {
        assert_eq!(meter_fraction(f32::NAN), 0.0);
    }

    /// Each decade of amplitude moves the fill further to the right.
    #[test]
    fn meter_fraction_increases_monotonically_with_amplitude() {
        let quiet = meter_fraction(0.001);
        let low = meter_fraction(0.01);
        let mid = meter_fraction(0.1);
        let full = meter_fraction(1.0);

        assert!(quiet < low, "{quiet} is not below {low}");
        assert!(low < mid, "{low} is not below {mid}");
        assert!(mid < full, "{mid} is not below {full}");
    }

    /// The fill fraction is always finite and inside 0.0..=1.0, including for
    /// degenerate inputs, because it is multiplied into the pill width.
    #[test]
    fn meter_fraction_stays_within_the_unit_range() {
        let amplitudes = [
            f32::MIN_POSITIVE,
            f32::MIN_POSITIVE * 0.5,
            3.16e-5, // -90 dBFS, below the meter floor
            0.001,
            0.01,
            0.1,
            0.5,
            0.999,
            1.0,
            1.5,
            100.0,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
            -1.0,
            0.0,
        ];

        for amplitude in amplitudes {
            let fraction = meter_fraction(amplitude);
            assert!(
                fraction.is_finite(),
                "amplitude {amplitude} produced a non-finite fraction {fraction}"
            );
            assert!(
                (0.0..=1.0).contains(&fraction),
                "amplitude {amplitude} produced {fraction}, outside 0.0..=1.0"
            );
        }
    }

    /// No reduction, and negative values, leave the GR bar empty: it only
    /// grows away from the center line.
    #[test]
    fn gr_fraction_zero_and_negative_db_are_empty() {
        assert_eq!(gr_fraction(0.0), 0.0);
        assert_eq!(gr_fraction(-3.0), 0.0);
    }

    /// Half of the meter's 24 dB range is 12 dB of reduction.
    #[test]
    fn gr_fraction_half_scale_is_twelve_db() {
        assert_close(gr_fraction(12.0), 0.5);
    }

    /// The bar is full at 24 dB and stays full beyond the meter maximum.
    #[test]
    fn gr_fraction_clamps_at_the_meter_maximum() {
        assert_eq!(gr_fraction(24.0), 1.0);
        assert_eq!(gr_fraction(30.0), 1.0);
    }

    /// A typical 6 dB of compression fills a quarter of the bar instead of the
    /// barely visible sliver the linear fill drew.
    #[test]
    fn gr_fraction_six_db_is_a_quarter() {
        assert_close(gr_fraction(6.0), 0.25);
    }

    /// A NaN reduction reads as no reduction rather than an invalid height.
    #[test]
    fn gr_fraction_nan_is_zero() {
        assert_eq!(gr_fraction(f32::NAN), 0.0);
    }

    /// More reduction never shrinks the bar, and every result stays finite and
    /// inside 0.0..=1.0.
    #[test]
    fn gr_fraction_is_monotonic_and_bounded() {
        let reductions_db = [
            0.0, 0.5, 1.0, 3.0, 6.0, 9.0, 12.0, 18.0, 23.9, 24.0, 30.0, 60.0,
        ];
        let mut previous = 0.0;

        for db in reductions_db {
            let fraction = gr_fraction(db);
            assert!(
                fraction.is_finite(),
                "{db} dB produced a non-finite fraction {fraction}"
            );
            assert!(
                (0.0..=1.0).contains(&fraction),
                "{db} dB produced {fraction}, outside 0.0..=1.0"
            );
            assert!(
                fraction >= previous,
                "{db} dB produced {fraction}, below the previous {previous}"
            );
            previous = fraction;
        }
    }
}
