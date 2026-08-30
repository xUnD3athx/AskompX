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

//! A thin wrapper around the Faust-generated DSP (`src/faust_dsp.rs`,
//! compiled from `dsp/askomp_dsp.dsp` with `faust -lang rust -double`).
//!
//! The compressor algorithm itself is exactly the one Faust generated for the
//! original plugin; this module only maps the plugin's parameters onto the
//! generated slider fields (using the same field mapping as the original C#
//! `AskompDsp` wrapper) and converts between the host's `f32` buffers and the
//! DSP's `f64` buffers.

use crate::faust_dsp::{mydsp, ParamIndex};

/// The Faust `ParamIndex` of every slider, in `dsp/askomp_dsp.dsp` declaration
/// order. Addressing the sliders through `set_param` keeps the adapter immune
/// to Faust renumbering the internal struct fields across regenerations.
const ATTACK: ParamIndex = ParamIndex(0);
const KNEE: ParamIndex = ParamIndex(1);
const NEGATIVE_THRESHOLD: ParamIndex = ParamIndex(2);
const POSITIVE_THRESHOLD: ParamIndex = ParamIndex(3);
const RATIO: ParamIndex = ParamIndex(4);
const RELEASE: ParamIndex = ParamIndex(5);
const STEREO_LINK: ParamIndex = ParamIndex(6);

#[derive(Debug, Clone, Copy)]
pub struct CompressorSettings {
    pub positive_threshold_db: f64,
    pub negative_threshold_db: f64,
    /// Attack time in milliseconds.
    pub attack_ms: f64,
    /// Release time in milliseconds.
    pub release_ms: f64,
    pub ratio: f64,
    /// The soft knee width in decibels.
    pub knee_db: f64,
    /// The stereo link amount: `0` compresses the channels independently,
    /// `1` fully links their envelopes.
    pub stereo_link: f64,
}

impl Default for CompressorSettings {
    fn default() -> Self {
        Self {
            positive_threshold_db: 0.0,
            negative_threshold_db: 0.0,
            attack_ms: 10.0,
            release_ms: 50.0,
            ratio: 2.0,
            knee_db: 3.0,
            stereo_link: 0.0,
        }
    }
}

/// The Faust DSP fixed to the stereo layout the plugin declares.
pub struct AskompDsp {
    dsp: mydsp,
}

impl AskompDsp {
    pub fn new(settings: &CompressorSettings, sample_rate: f64) -> Self {
        let mut dsp = mydsp::new();
        dsp.init(sample_rate as i32);
        Self::apply(&mut dsp, settings);

        Self { dsp }
    }

    /// Updates the Faust sliders from the plugin's parameters.
    fn apply(dsp: &mut mydsp, settings: &CompressorSettings) {
        dsp.set_param(NEGATIVE_THRESHOLD, settings.negative_threshold_db);
        dsp.set_param(POSITIVE_THRESHOLD, settings.positive_threshold_db);
        dsp.set_param(ATTACK, settings.attack_ms);
        dsp.set_param(RELEASE, settings.release_ms);
        dsp.set_param(RATIO, settings.ratio);
        dsp.set_param(KNEE, settings.knee_db);
        dsp.set_param(STEREO_LINK, settings.stereo_link);
    }

    pub fn update_settings(&mut self, settings: &CompressorSettings) {
        Self::apply(&mut self.dsp, settings);
    }

    /// Reinitializes the DSP for a new sample rate, keeping the current
    /// parameter values.
    pub fn set_sample_rate(&mut self, settings: &CompressorSettings, sample_rate: f64) {
        self.dsp.init(sample_rate as i32);
        Self::apply(&mut self.dsp, settings);
    }

    /// Processes a block of stereo audio. The buffers are channel-major.
    pub fn compute(&mut self, count: usize, inputs: &[f64], outputs: &mut [f64]) {
        let input_left = &inputs[..count];
        let input_right = &inputs[count..count * 2];
        let (output_left, output_right) = outputs.split_at_mut(count);

        self.dsp.compute(count, &[input_left, input_right], &mut [output_left, output_right]);
    }

    pub fn reset(&mut self) {
        // `instance_clear()` resets the delay lines and filter states, which
        // for this DSP means the envelope followers.
        self.dsp.instance_clear();
    }
}

/// Converts a linear amplitude to decibels, mirroring
/// `AudioPluginParameter.LinearToDB` from the original plugin.
#[inline]
pub fn linear_to_db(linear: f64) -> f64 {
    20.0 * linear.max(f64::MIN_POSITIVE).log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f64 = 48000.0;

    fn default_settings() -> CompressorSettings {
        CompressorSettings {
            positive_threshold_db: -12.0,
            negative_threshold_db: -12.0,
            attack_ms: 0.0,
            release_ms: 100.0,
            ratio: 2.0,
            knee_db: 3.0,
            stereo_link: 0.0,
        }
    }

    /// Feeds a constant stereo level through the compressor and returns the
    /// steady state output level of the left channel.
    fn steady_state_level(settings: &CompressorSettings, input_level: f64) -> f64 {
        let mut dsp = AskompDsp::new(settings, SAMPLE_RATE);

        let mut input = vec![0.0f64; 2];
        let mut output = vec![0.0f64; 2];

        let mut last = 0.0;
        for _ in 0..48000 {
            input[0] = input_level;
            input[1] = input_level;
            dsp.compute(1, &input, &mut output);
            last = output[0];
        }

        last
    }

    #[test]
    fn passes_levels_below_the_threshold() {
        let settings = default_settings();
        // 6 dB below the -12 dB threshold, so well outside the knee.
        let output = steady_state_level(&settings, 10f64.powf(-18.0 / 20.0));
        let output_db = linear_to_db(output);

        assert!((output_db - -18.0).abs() < 0.1, "got {output_db} dB");
    }

    #[test]
    fn applies_the_ratio_above_the_threshold() {
        let settings = default_settings();
        // 12 dB over the -12 dB threshold at 2:1 compresses to 6 dB over.
        let output = steady_state_level(&settings, 10f64.powf(0.0 / 20.0));
        let output_db = linear_to_db(output);

        assert!((output_db - -6.0).abs() < 0.2, "got {output_db} dB");
    }

    #[test]
    fn positive_and_negative_halves_compress_independently() {
        // The negative half gets a much lower threshold, so a symmetric input
        // signal comes out asymmetric.
        let settings = CompressorSettings {
            positive_threshold_db: -12.0,
            negative_threshold_db: -36.0,
            ..default_settings()
        };

        let positive_out = steady_state_level(&settings, 0.5);
        let negative_out = steady_state_level(&settings, -0.5);

        assert!(
            linear_to_db(positive_out.abs()) > linear_to_db(negative_out.abs()) + 1.0,
            "positive side ({positive_out}) should be less compressed than the \
             negative side ({negative_out})"
        );
    }

    #[test]
    fn knee_matches_the_gain_computer_formula() {
        // `gain_computer` from Faust's compressors.lib, 1.5 dB over the
        // -12 dB threshold at 2:1:
        //
        //   knee 0:  reduction = strength * (level - thresh)
        //   knee 24: reduction = strength * (level - thresh + knee/2)^2 / (2 * knee)
        //
        // Pinning both values also catches the slider fields being swapped.
        let one_and_a_half_over = 10f64.powf(-10.5 / 20.0);

        let hard = CompressorSettings {
            knee_db: 0.0,
            ..default_settings()
        };
        let soft = CompressorSettings {
            knee_db: 24.0,
            ..default_settings()
        };

        let hard_out = linear_to_db(steady_state_level(&hard, one_and_a_half_over).abs());
        let soft_out = linear_to_db(steady_state_level(&soft, one_and_a_half_over).abs());

        assert!(
            (hard_out - -11.25).abs() < 0.05,
            "hard knee output {hard_out} dB, expected -11.25 dB"
        );
        assert!(
            (soft_out - -12.398).abs() < 0.1,
            "24 dB knee output {soft_out} dB, expected -12.398 dB"
        );
    }

    #[test]
    fn stereo_link_couples_the_channel_envelopes() {
        // A loud left channel and a quiet right channel: with the channels
        // linked the right channel must be compressed down by the left
        // channel's envelope, with the channels unlinked it passes freely.
        let linked = CompressorSettings {
            stereo_link: 1.0,
            ..default_settings()
        };
        let unlinked = CompressorSettings {
            stereo_link: 0.0,
            ..default_settings()
        };

        let mut linked_dsp = AskompDsp::new(&linked, SAMPLE_RATE);
        let mut unlinked_dsp = AskompDsp::new(&unlinked, SAMPLE_RATE);

        // One stereo sample: a loud left and a quiet right channel.
        let input = vec![0.5f64, 0.001];
        let mut output = vec![0.0f64; 2];
        let mut linked_right = 0.0;
        let mut unlinked_right = 0.0;
        for _ in 0..48000 {
            linked_dsp.compute(1, &input, &mut output);
            linked_right = output[1];

            unlinked_dsp.compute(1, &input, &mut output);
            unlinked_right = output[1];
        }

        // The left channel sits 6 dB over the threshold at 2:1, so the linked
        // right channel inherits a 3 dB (70.8%) reduction.
        assert!(
            linked_right < unlinked_right * 0.75,
            "linked right channel ({linked_right}) should be dragged down by \
             the left channel's envelope, unlinked ({unlinked_right}) should not"
        );
    }

    #[test]
    fn processes_blocks_consistently() {
        let settings = default_settings();
        let mut dsp = AskompDsp::new(&settings, SAMPLE_RATE);

        let block_length = 256;
        let input = vec![0.5f64; block_length * 2];
        let mut output = vec![0.0f64; block_length * 2];
        for _ in 0..100 {
            dsp.compute(block_length, &input, &mut output);
        }

        // The same total number of samples processed one sample at a time.
        let mut per_sample_dsp = AskompDsp::new(&settings, SAMPLE_RATE);
        let single_input = vec![0.5f64; 2];
        let mut single_output = vec![0.0f64; 2];
        let mut per_sample = 0.0;
        for _ in 0..100 * block_length {
            per_sample_dsp.compute(1, &single_input, &mut single_output);
            per_sample = single_output[0];
        }

        assert!(
            (output[block_length - 1] - per_sample).abs() < 1e-12,
            "block processing ({}) should match sample processing ({per_sample})",
            output[block_length - 1]
        );
    }
}
