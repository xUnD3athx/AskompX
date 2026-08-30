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

use crate::faust_dsp::mydsp;

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

    /// Updates the Faust sliders from the plugin's parameters, mirroring the
    /// field mapping of the original C# `AskompDsp` wrapper:
    ///
    /// - `fVslider0` = Negative Threshold
    /// - `fVslider1` = Positive Threshold
    /// - `fHslider0` = Release
    /// - `fHslider1` = Attack
    /// - `fHslider2` = Ratio
    /// - `fHslider4` = Knee
    /// - `fHslider5` = Stereo Link
    /// - `fHslider3` = Trim (not exposed, stays at its 0 dB default)
    fn apply(dsp: &mut mydsp, settings: &CompressorSettings) {
        dsp.fVslider0 = settings.negative_threshold_db as _;
        dsp.fVslider1 = settings.positive_threshold_db as _;
        dsp.fHslider0 = settings.release_ms as _;
        dsp.fHslider1 = settings.attack_ms as _;
        dsp.fHslider2 = settings.ratio as _;
        dsp.fHslider4 = settings.knee_db as _;
        dsp.fHslider5 = settings.stereo_link as _;
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
    fn processes_blocks_consistently() {
        let settings = default_settings();
        let mut dsp = AskompDsp::new(&settings, SAMPLE_RATE);

        let block_length = 256;
        let mut input = vec![0.5f64; block_length * 2];
        let mut output = vec![0.0f64; block_length * 2];
        for _ in 0..100 {
            dsp.compute(block_length, &input, &mut output);
        }

        // The same total number of samples processed one sample at a time.
        let mut per_sample_dsp = AskompDsp::new(&settings, SAMPLE_RATE);
        let mut single_input = vec![0.5f64; 2];
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
