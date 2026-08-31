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
                let mix_out = ((compressed_l + compressed_r) / 2.0) as f64;

                // The gain reduction applied to either half-wave, as a linear
                // amplitude delta.
                positive_compression = positive_compression
                    .max((mix_in.max(0.0) - mix_out.max(0.0)).abs());
                negative_compression = negative_compression
                    .max((mix_in.min(0.0) - mix_out.min(0.0)).abs());

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
            dsp::linear_to_db(1.0 - positive_compression).abs() as f32;
        let new_negative_db =
            dsp::linear_to_db(1.0 - negative_compression).abs() as f32;
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
