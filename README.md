# EasySound V1.2

Created by NorthStar.

A lightweight real-time audio processor for Windows (and Linux/macOS).

Built in **Rust** with native DSP — no Electron, no browser runtime. One small executable, low CPU/RAM.

## Effects (20)

| | | | |
| --- | --- | --- | --- |
| Autogain | Bass Enhancer | Bass Loudness | Equalizer (16-band) |
| Filter | Compressor | Multiband Compressor | Exciter |
| Crystalizer | Deesser | Delay | Reverb |
| Pitch | Stereo Tools | Crossfeed | Loudness |
| Noise Gate | Multiband Gate | Maximizer | Limiter |

## Install / run

```bash
cargo build --release
```

Then run:

Windows 10/11 (build on Windows): `cargo build --release` → `target\release\easysound.exe`

Copy the binary anywhere and double-click it. Pin it to the Start Menu or taskbar — no Node/npm required.

### Windows system-wide audio

1. Install [VB-Audio Virtual Cable](https://vb-audio.com/Cable/) or VoiceMeeter.
2. Set app playback to **Cable Input**.
3. Point EasySound’s input at **Cable Output** (or make it the default recording device).
4. Choose your speakers or headphones as EasySound’s **Output**, then click **Listen**.
5. Enable **Equalizer** and adjust the vertical faders while audio plays. Click **Listen: On** to stop monitoring.

Use **Demo tone** to try the effects without configuring an input. Use headphones when listening to a microphone to avoid feedback.

## Save and load settings

Use **Save Settings** to export your EQ and effects to a JSON file. Use **Load Settings** to restore them. Loaded effects take effect immediately, including during Listen.

## Develop

```bash
cargo run
```

## Why Rust?

Native in-process DSP and a small immediate-mode UI (`egui` + `cpal`) keep latency and system requirements low compared to Electron/web wrappers.
