# AskompX (nih-plug rewrite)

AskompX is an **asymmetrical** compressor.
It compresses the positive signal (above zero) and the negative signal (below zero) separately.
Using this plugin can keep your audio centered and naturally gain perceived loudness.

You can also unlink the positive and negative thresholds and you will hear harmonics added to
your audio. This is because the positive and negative signal are compressed at different
thresholds. An odd function is origin-symmetric, and compressing asymmetrically destroys the
signal symmetry, so even harmonics are added, which makes the tone more tube-style. This sounds
similar to asymmetric distortion, but smoother and more natural.

This directory is a full rewrite of the original C#/AudioPlugSharp/WPF plugin in **Rust** using
[nih-plug](https://github.com/robbert-vdh/nih-plug). The DSP is the original Faust program
(`dsp/askomp_dsp.dsp`) compiled directly with the Faust Rust backend:

```
faust -lang rust -double dsp/askomp_dsp.dsp -o src/faust_dsp.rs
```

`src/faust_dsp.rs` is that generated file with only minimal mechanical patches (type aliases,
field visibility) so it compiles standalone — the algorithm itself is exactly what Faust
generates. `src/dsp.rs` is a thin adapter that maps the plugin parameters onto the generated
slider fields (same field mapping as the original C# `AskompDsp` wrapper). The Vizia editor
mirrors the original WPF interface: waveform display, draggable threshold lines, output meters,
gain reduction meters, and the knob row.

## Features

- VST3 + CLAP output on Windows (and Linux/macOS, unlike the original which was Windows-only)
- Positive/negative half-wave compressors with independent envelopes and a fixed 3 dB knee
- Parameters: Positive Threshold, Negative Threshold (both -36…0 dB), Attack (0…50 ms),
  Release (20…200 ms), Ratio (1:1…20:1), Link
- Link synchronizes the two thresholds while dragging the threshold lines
- Waveform display (input and output) with the two draggable threshold lines

## Build

You need a Rust toolchain (stable). Then, from this directory:

```
cargo xtask bundle askompx --release
```

The bundled plugin is written to `target/bundled/askompx.vst3`. Copy that folder to your VST3
directory (`C:\Program Files\Common Files\VST3`). No additional runtime fonts or DLLs are needed,
unlike the original.

The same command also writes `askompx.clap` and a standalone `askompx.exe` alongside the VST3
bundle.

To preview the editor outside of a DAW, run the standalone binary:

```
cargo run --release
```

(The standalone wrapper can panic on some audio devices due to a buffer size
mismatch in nih-plug's cpal backend; this only affects the standalone build,
not the VST3/CLAP plugins.)

## Tests

The Faust DSP is covered by unit tests:

```
cargo test
```

They verify the threshold, the 2:1 ratio above the knee, the independent positive/negative
half-wave compression, and that block processing matches sample-by-sample processing.

## Layout

- `src/faust_dsp.rs` — the Faust-generated DSP (`faust -lang rust -double`)
- `src/dsp.rs` — adapter mapping the plugin's parameters onto the generated sliders
- `src/lib.rs` — plugin definition, parameters, audio processing, meter/waveform shared state
- `src/editor.rs` — the Vizia editor
- `xtask/` — the bundling tooling from nih-plug

## License

GPL-3.0 — see [LICENSE](LICENSE). DSP & Plugin: UnD3ath, © 2026 UnD3ath.
