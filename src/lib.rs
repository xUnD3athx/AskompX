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

use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

mod dsp;
mod editor;
mod faust_dsp;

pub use dsp::CompressorSettings;

/// The resolution of the waveform display.
pub const WAVE_SIZE: usize = 128;

/// Waveform amplitudes below this level are rendered as silence, mirroring the
/// original plugin's `-36 dB` gate.
const WAVE_GATE_LINEAR: f32 = 0.0158489319;

#[derive(Params)]
pub struct AskompParams {
    /// The threshold for the positive (above zero) half-wave, in dB.
    #[id = "posTh"]
    pub positive_threshold: FloatParam,

    /// The threshold for the negative (below zero) half-wave, in dB.
    #[id = "negTh"]
    pub negative_threshold: FloatParam,

    /// The compressor attack time in milliseconds.
    #[id = "attack"]
    pub attack: FloatParam,

    /// The compressor release time in milliseconds.
    #[id = "release"]
    pub release: FloatParam,

    /// The compressor ratio.
    #[id = "ratio"]
    pub ratio: FloatParam,

    /// The soft knee width in decibels.
    #[id = "knee"]
    pub knee: FloatParam,

    /// The stereo link amount. `0%` compresses the channels independently,
    /// `100%` fully links their envelopes.
    #[id = "stLink"]
    pub stereo_link: FloatParam,

    /// The sidechain high-pass cutoff. The compressor detects on the
    /// high-passed signal while gain reducing the full-range signal.
    #[id = "schp"]
    pub sidechain_hp: FloatParam,

    /// Whether the two thresholds should move together.
    #[id = "link"]
    pub link: BoolParam,

    /// Whether to output the difference between the processed and the input
    /// signal instead of the processed signal.
    #[id = "delta"]
    pub delta: BoolParam,
}

impl Default for AskompParams {
    fn default() -> Self {
        Self {
            positive_threshold: FloatParam::new(
                "Positive Threshold",
                0.0,
                FloatRange::Linear {
                    min: -36.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB")
            .with_step_size(0.1),

            negative_threshold: FloatParam::new(
                "Negative Threshold",
                0.0,
                FloatRange::Linear {
                    min: -36.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB")
            .with_step_size(0.1),

            attack: FloatParam::new("Attack", 10.0, FloatRange::Linear { min: 0.0, max: 50.0 })
                .with_unit(" ms")
                .with_step_size(0.1),

            release: FloatParam::new("Release", 50.0, FloatRange::Linear { min: 20.0, max: 200.0 })
                .with_unit(" ms")
                .with_step_size(1.0),

            ratio: FloatParam::new("Ratio", 2.0, FloatRange::Linear { min: 1.0, max: 20.0 })
                .with_unit(" : 1")
                .with_step_size(1.0),

            knee: FloatParam::new("Knee", 3.0, FloatRange::Linear { min: 0.0, max: 24.0 })
                .with_unit(" dB")
                .with_step_size(0.1),

            stereo_link: FloatParam::new(
                "Stereo Link",
                0.0,
                FloatRange::Linear { min: 0.0, max: 100.0 },
            )
            .with_unit(" %")
            .with_step_size(1.0),

            sidechain_hp: FloatParam::new(
                "Sidechain HP",
                20.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 500.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" Hz")
            .with_step_size(1.0),

            link: BoolParam::new("Link", true),

            delta: BoolParam::new("Delta", false),
        }
    }
}

/// Data shared between the audio thread and the editor, for meters and the
/// waveform display.
pub struct AskompShared {
    /// The plugin's parameters.
    pub params: Arc<AskompParams>,

    /// The output peak level of the left channel.
    pub output_peak_left: AtomicF32,
    /// The output peak level of the right channel.
    pub output_peak_right: AtomicF32,
    /// The positive gain reduction in linear amplitude.
    pub positive_compression: AtomicF32,
    /// The negative gain reduction in linear amplitude.
    pub negative_compression: AtomicF32,
    /// The positive gain reduction in decibels.
    pub positive_compression_db: AtomicF32,
    /// The negative gain reduction in decibels.
    pub negative_compression_db: AtomicF32,

    /// The stereo-mixed input waveform, newest sample last.
    pub wave_in: Box<[AtomicF32]>,
    /// The stereo-mixed output waveform, newest sample last.
    pub wave_out: Box<[AtomicF32]>,

    /// How many process blocks pass before the waveform display scrolls by one
    /// column. This was one of the few user-configurable settings in the
    /// original UI.
    pub wave_step: AtomicUsize,
}

impl AskompShared {
    fn new(params: Arc<AskompParams>) -> Self {
        Self {
            params,
            output_peak_left: AtomicF32::new(0.0),
            output_peak_right: AtomicF32::new(0.0),
            positive_compression: AtomicF32::new(0.0),
            negative_compression: AtomicF32::new(0.0),
            positive_compression_db: AtomicF32::new(0.0),
            negative_compression_db: AtomicF32::new(0.0),
            wave_in: std::iter::repeat_with(|| AtomicF32::new(0.0))
                .take(WAVE_SIZE)
                .collect(),
            wave_out: std::iter::repeat_with(|| AtomicF32::new(0.0))
                .take(WAVE_SIZE)
                .collect(),
            // The original defaults to a step of 8 (`_step = 8`).
            wave_step: AtomicUsize::new(8),
        }
    }
}

/// The stereo mix of the compressed (pre-delta) signal.
fn compressed_mix(left: f32, right: f32) -> f64 {
    (left as f64 + right as f64) / 2.0
}

/// The stereo mix that is actually written to the output buffer. In delta mode
/// this is the difference between the compressed and the input signal.
fn output_mix(input_mix: f64, compressed_mix: f64, delta: bool) -> f64 {
    if delta {
        compressed_mix - input_mix
    } else {
        compressed_mix
    }
}

/// The largest gain reduction the meters display, in dB.
const MAX_GR_DB: f32 = 60.0;

/// Gain reduction in dB for one half-wave, derived from the input and
/// compressed-output peak amplitudes of that half-wave.
fn gain_reduction_db(input_peak: f64, output_peak: f64) -> f32 {
    if !input_peak.is_finite() || !output_peak.is_finite() || input_peak <= 0.0 {
        return 0.0;
    }
    if output_peak <= 0.0 {
        return MAX_GR_DB;
    }

    ((20.0 * (input_peak / output_peak).log10()) as f32).clamp(0.0, MAX_GR_DB)
}

/// The `Askomp` asymmetrical compressor.
pub struct Askomp {
    params: Arc<AskompParams>,
    shared: Arc<AskompShared>,
    editor_state: Arc<ViziaState>,

    dsp: Option<dsp::AskompDsp>,
    /// The sample rate the DSP was last initialized with.
    dsp_sample_rate: f64,

    /// Counts process blocks to slow down the waveform display's scrolling.
    wave_step_counter: usize,
}

impl Default for Askomp {
    fn default() -> Self {
        let params: Arc<AskompParams> = Arc::default();
        let shared = Arc::new(AskompShared::new(params.clone()));
        // Same dimensions as the original WPF editor.
        let editor_state = ViziaState::new(|| (500, 400));

        Self {
            params,
            shared,
            editor_state,
            dsp: None,
            dsp_sample_rate: 48000.0,
            wave_step_counter: 0,
        }
    }
}

impl ClapPlugin for Askomp {
    const CLAP_ID: &'static str = "top.aqua-sounds.askompx";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("An asymmetrical compressor");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Compressor,
    ];
}

impl Vst3Plugin for Askomp {
    const VST3_CLASS_ID: [u8; 16] = *b"AquaSoundsAskomX";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

impl Plugin for Askomp {
    const NAME: &'static str = "AskompX";
    const VENDOR: &'static str = "Aqua Sounds";
    const URL: &'static str = "https://aqua-sounds.top";
    const EMAIL: &'static str = "und3ath@outlook.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn reset(&mut self) {
        if let Some(dsp) = &mut self.dsp {
            dsp.reset();
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let num_channels = buffer.channels() as usize;
        if num_channels == 0 {
            return ProcessStatus::Normal;
        }

        let sample_rate = _context.transport().sample_rate as f64;

        // The original plugin updates its Faust DSP's parameters once per
        // process block, so this mirrors that behavior.
        let settings = CompressorSettings {
            positive_threshold_db: self.params.positive_threshold.value() as f64,
            negative_threshold_db: self.params.negative_threshold.value() as f64,
            attack_ms: self.params.attack.value() as f64,
            release_ms: self.params.release.value() as f64,
            ratio: self.params.ratio.value() as f64,
            knee_db: self.params.knee.value() as f64,
            stereo_link: (self.params.stereo_link.value() / 100.0) as f64,
            sidechain_hp_hz: self.params.sidechain_hp.value() as f64,
        };

        let dsp = self
            .dsp
            .get_or_insert_with(|| dsp::AskompDsp::new(&settings, sample_rate));
        if self.dsp_sample_rate != sample_rate {
            dsp.set_sample_rate(&settings, sample_rate);
            self.dsp_sample_rate = sample_rate;
        } else {
            dsp.update_settings(&settings);
        }

        // Per-block meter maxima, matching the original's process loop.
        let delta = self.params.delta.value();
        let mut output_peak_left = 0.0f32;
        let mut output_peak_right = 0.0f32;
        let mut positive_compression = 0.0f64;
        let mut negative_compression = 0.0f64;
        // The input and compressed-output peaks per half-wave, used for the
        // dB gain reduction readouts.
        let mut positive_input_peak = 0.0f64;
        let mut positive_output_peak = 0.0f64;
        let mut negative_input_peak = 0.0f64;
        let mut negative_output_peak = 0.0f64;

        // The signed peak of the stereo mix over this block, used by the
        // waveform display.
        let mut mix_in_peak = 0.0f64;
        let mut mix_out_peak = 0.0f64;

        let mut wave_step_counter = self.wave_step_counter;
        let wave_step = self.shared.wave_step.load(Ordering::Relaxed);
        let mut waveform_shifted = false;
        if wave_step_counter > wave_step {
            // Scroll the waveform displays one column to the left.
            let (wave_in, wave_out) = (&self.shared.wave_in, &self.shared.wave_out);
            for i in 0..WAVE_SIZE - 1 {
                wave_in[i].store(
                    wave_in[i + 1].load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
                wave_out[i].store(
                    wave_out[i + 1].load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
            }
            wave_step_counter = 0;
            waveform_shifted = true;
        }

        let channel_slices = buffer.as_slice();
        let block_length = channel_slices.first().map(|c| c.len()).unwrap_or(0);

        if num_channels == 2 && block_length > 0 {
            // Convert the block to the DSP's f64 channel-major buffers, run
            // the Faust-generated compute function once per block (the
            // original also updates its parameters per block), and convert
            // back.
            let mut input_f64 = vec![0.0f64; block_length * 2];
            let mut output_f64 = vec![0.0f64; block_length * 2];
            for sample_idx in 0..block_length {
                input_f64[sample_idx] = channel_slices[0][sample_idx] as f64;
                input_f64[block_length + sample_idx] = channel_slices[1][sample_idx] as f64;
            }

            dsp.compute(block_length, &input_f64, &mut output_f64);

            for sample_idx in 0..block_length {
                let in_l = input_f64[sample_idx];
                let in_r = input_f64[block_length + sample_idx];
                let compressed_l = output_f64[sample_idx] as f32;
                let compressed_r = output_f64[block_length + sample_idx] as f32;

                // In delta mode the plugin outputs the difference signal, so
                // only what the compressor actually did is audible. The gain
                // reduction meters always measure the compressor's action on
                // the full signal, not the delta output.
                let (out_l, out_r) = if delta {
                    (
                        compressed_l - in_l as f32,
                        compressed_r - in_r as f32,
                    )
                } else {
                    (compressed_l, compressed_r)
                };

                channel_slices[0][sample_idx] = out_l;
                channel_slices[1][sample_idx] = out_r;

                let mix_in = (in_l + in_r) / 2.0;
                // The gain reduction meters always measure the compressor's
                // action, so they are compared against the pre-delta signal.
                let compressed = compressed_mix(compressed_l, compressed_r);
                // The waveform display shows what actually leaves the plugin.
                let mix_out = output_mix(mix_in, compressed, delta);

                // The gain reduction applied to either half-wave, as a linear
                // amplitude delta.
                positive_compression = positive_compression
                    .max((mix_in.max(0.0) - compressed.max(0.0)).abs());
                negative_compression = negative_compression
                    .max((mix_in.min(0.0) - compressed.min(0.0)).abs());

                positive_input_peak = positive_input_peak.max(mix_in.max(0.0));
                positive_output_peak = positive_output_peak.max(compressed.max(0.0));
                negative_input_peak = negative_input_peak.max(-mix_in.min(0.0));
                negative_output_peak = negative_output_peak.max(-compressed.min(0.0));

                // The original only looks at the positive half for these
                // meters.
                output_peak_left = output_peak_left.max(out_l);
                output_peak_right = output_peak_right.max(out_r);

                if mix_in.abs() > mix_in_peak.abs() {
                    mix_in_peak = mix_in;
                }
                if mix_out.abs() > mix_out_peak.abs() {
                    mix_out_peak = mix_out;
                }
            }
        }

        // Update the waveform display on the last sample of the block.
        if waveform_shifted {
            let wave_gate = WAVE_GATE_LINEAR as f64;
            let wave_in_value = if mix_in_peak.abs() < wave_gate {
                0.0
            } else {
                mix_in_peak
            };
            let wave_out_value = if mix_out_peak.abs() < wave_gate {
                0.0
            } else {
                mix_out_peak
            };
            self.shared
                .wave_in
                .last()
                .unwrap()
                .store(wave_in_value as f32, Ordering::Relaxed);
            self.shared
                .wave_out
                .last()
                .unwrap()
                .store(wave_out_value as f32, Ordering::Relaxed);
        }

        self.wave_step_counter = wave_step_counter + 1;

        // Meter smoothing, applied once per process block exactly like the
        // original's UI updates (`Lerp(new, old, 0.95)` and the 0.995 hold for
        // the numeric gain reduction values). The UI reads the smoothed values
        // directly.
        let smooth_meter = |old: f32, new: f32| (old * 0.95) + (new * 0.05);
        let old_peak_left = self.shared.output_peak_left.load(Ordering::Relaxed);
        let old_peak_right = self.shared.output_peak_right.load(Ordering::Relaxed);
        self.shared
            .output_peak_left
            .store(smooth_meter(old_peak_left, output_peak_left), Ordering::Relaxed);
        self.shared
            .output_peak_right
            .store(smooth_meter(old_peak_right, output_peak_right), Ordering::Relaxed);

        let old_positive = self.shared.positive_compression.load(Ordering::Relaxed);
        let old_negative = self.shared.negative_compression.load(Ordering::Relaxed);
        self.shared
            .positive_compression
            .store(smooth_meter(old_positive, positive_compression as f32), Ordering::Relaxed);
        self.shared
            .negative_compression
            .store(smooth_meter(old_negative, negative_compression as f32), Ordering::Relaxed);

        // The original displays the absolute decibel value of the gain
        // reduction, holding the peak with a slow 0.995 release.
        let new_positive_db =
            gain_reduction_db(positive_input_peak, positive_output_peak);
        let new_negative_db =
            gain_reduction_db(negative_input_peak, negative_output_peak);
        let smooth_gr = |old: f32, new: f32| {
            if old > new {
                (old * 0.995) + (new * 0.005)
            } else {
                new
            }
        };
        let old_positive_db = self.shared.positive_compression_db.load(Ordering::Relaxed);
        let old_negative_db = self.shared.negative_compression_db.load(Ordering::Relaxed);
        self.shared
            .positive_compression_db
            .store(smooth_gr(old_positive_db, new_positive_db), Ordering::Relaxed);
        self.shared
            .negative_compression_db
            .store(smooth_gr(old_negative_db, new_negative_db), Ordering::Relaxed);

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.shared.params.clone(),
            self.shared.clone(),
            self.editor_state.clone(),
        )
    }
}

nih_export_clap!(Askomp);
nih_export_vst3!(Askomp);

#[cfg(test)]
mod tests {
    use super::*;

    /// Tolerance for comparisons where `f32` inputs were widened to `f64`.
    const EPSILON: f64 = 1e-6;

    fn assert_close(actual: f64, expected: f64) {
        let difference = (actual - expected).abs();
        assert!(
            difference < EPSILON,
            "expected {expected}, got {actual} (difference {difference})"
        );
    }

    /// Tolerance for gain-reduction values, which are `f32` and involve `log10`.
    const DB_EPSILON: f32 = 1e-3;

    fn assert_db_close(actual: f32, expected: f32) {
        let difference = (actual - expected).abs();
        assert!(
            difference < DB_EPSILON,
            "expected {expected} dB, got {actual} dB (difference {difference})"
        );
    }

    #[test]
    fn compressed_mix_averages_the_two_channels() {
        assert_close(compressed_mix(0.5, 0.1), 0.3);
        assert_eq!(compressed_mix(-0.5, 0.5), 0.0);
        assert_eq!(compressed_mix(1.0, 1.0), 1.0);
    }

    #[test]
    fn non_delta_output_equals_the_compressed_mix() {
        let compressed = compressed_mix(0.8, 0.2);

        assert_eq!(output_mix(0.9, compressed, false), compressed);
        assert_eq!(output_mix(-0.3, compressed, false), compressed);
    }

    #[test]
    fn delta_output_equals_compressed_minus_input() {
        let output = output_mix(0.5, 0.25, true);

        assert_eq!(output, -0.25);
    }

    #[test]
    fn delta_output_is_the_average_of_the_per_channel_differences() {
        let (in_l, in_r) = (1.0f32, 0.0f32);
        let (comp_l, comp_r) = (0.4f32, 0.0f32);

        let input_mix = compressed_mix(in_l, in_r);
        let compressed = compressed_mix(comp_l, comp_r);
        let output = output_mix(input_mix, compressed, true);

        let average_of_per_channel_differences =
            ((comp_l as f64 - in_l as f64) + (comp_r as f64 - in_r as f64)) / 2.0;

        assert_close(output, average_of_per_channel_differences);
        assert_close(output, -0.3);
    }

    #[test]
    fn delta_output_is_silent_when_nothing_was_compressed() {
        for (left, right) in [(0.0f32, 0.0f32), (0.25, -0.75), (1.0, 1.0), (-0.5, 0.125)] {
            let input_mix = compressed_mix(left, right);
            // Nothing was compressed, so the compressed signal is the input.
            let compressed = compressed_mix(left, right);

            assert_eq!(output_mix(input_mix, compressed, true), 0.0);
        }
    }

    #[test]
    fn delta_of_symmetric_input_and_compression_is_negative_quarter() {
        let input_mix = compressed_mix(0.5, 0.5);
        let compressed = compressed_mix(0.25, 0.25);

        assert_eq!(input_mix, 0.5);
        assert_eq!(compressed, 0.25);
        assert_eq!(output_mix(input_mix, compressed, true), -0.25);
    }

    #[test]
    fn gain_reduction_of_unity_input_to_half_output_is_six_db() {
        // 20 * log10(1.0 / 0.5) = 20 * log10(2) ~= 6.0206 dB.
        assert_db_close(gain_reduction_db(1.0, 0.5), 6.0206);
    }

    #[test]
    fn gain_reduction_of_half_input_to_quarter_output_is_six_db() {
        // Same ratio as 1.0 -> 0.5, so the same 20 * log10(2) dB.
        assert_db_close(gain_reduction_db(0.5, 0.25), 6.0206);
    }

    #[test]
    fn gain_reduction_of_silent_output_is_clamped_to_max() {
        let reduction = gain_reduction_db(1.0, 0.0);

        assert_eq!(reduction, MAX_GR_DB);
        assert_eq!(reduction, 60.0);
    }

    #[test]
    fn gain_reduction_of_unchanged_level_is_zero() {
        assert_eq!(gain_reduction_db(1.0, 1.0), 0.0);
    }

    #[test]
    fn gain_reduction_of_louder_output_is_zero() {
        // The output is louder than the input, so there is nothing to reduce.
        assert_eq!(gain_reduction_db(0.5, 1.0), 0.0);
    }

    #[test]
    fn gain_reduction_of_silence_is_zero() {
        assert_eq!(gain_reduction_db(0.0, 0.0), 0.0);
    }

    #[test]
    fn gain_reduction_of_hot_compressed_input_is_clamped_to_max() {
        // 2.0 / 0.0019 is ~1052.6 (about 60.44 dB), just past the display limit.
        // The old `linear_to_db(1.0 - compression).abs()` path reported thousands
        // of dB here because the linear difference went negative.
        let reduction = gain_reduction_db(2.0, 0.0019);

        assert_eq!(reduction, MAX_GR_DB);
        assert_eq!(reduction, 60.0);
    }

    #[test]
    fn gain_reduction_of_non_finite_values_is_zero() {
        assert_eq!(gain_reduction_db(f64::NAN, 0.5), 0.0);
        assert_eq!(gain_reduction_db(1.0, f64::NAN), 0.0);
        assert_eq!(gain_reduction_db(f64::INFINITY, 0.5), 0.0);
    }

    #[test]
    fn gain_reduction_stays_within_display_range() {
        let cases = [
            (1.0, 1.0),
            (1.0, 0.5),
            (1.0, 0.001),
            (1.0, 0.0),
            (0.5, 0.25),
            (0.75, 0.9),
            (2.0, 0.0019),
            (1e-9, 1e-12),
            (0.1, 0.0),
            (0.0, 0.0),
        ];

        for (input_peak, output_peak) in cases {
            let reduction = gain_reduction_db(input_peak, output_peak);

            assert!(
                reduction.is_finite(),
                "gain reduction must be finite for {input_peak} -> {output_peak}, got {reduction}"
            );
            assert!(
                (0.0..=MAX_GR_DB).contains(&reduction),
                "gain reduction {reduction} dB out of range for {input_peak} -> {output_peak}"
            );
        }
    }

    #[test]
    fn gain_reduction_grows_with_more_reduction() {
        // Halving the output halves the amplitude once more, so 1.0 -> 0.25 must
        // report more reduction than 1.0 -> 0.5.
        let halved = gain_reduction_db(1.0, 0.5);
        let quartered = gain_reduction_db(1.0, 0.25);

        assert!(
            halved < quartered,
            "1.0 -> 0.5 ({halved} dB) should be less than 1.0 -> 0.25 ({quartered} dB)"
        );
    }
}
