use crate::dsp::{EffectChain, EffectParams, SpectrumAnalyzer};
use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct MeterSnapshot {
    pub input_peak: f32,
    pub output_peak: f32,
    pub spectrum: Vec<f32>,
    pub running: bool,
    pub listening: bool,
    pub sample_rate: u32,
}

struct Shared {
    params: EffectParams,
    params_dirty: bool,
    input_peak: f32,
    output_peak: f32,
    spectrum: Vec<f32>,
    generate_demo: bool,
    demo_phase: f32,
    demo_phase2: f32,
}

pub struct AudioEngine {
    shared: Arc<Mutex<Shared>>,
    running: Arc<AtomicBool>,
    _input_stream: Option<Stream>,
    _output_stream: Option<Stream>,
    sample_rate: u32,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared {
                params: EffectParams::default(),
                params_dirty: true,
                input_peak: 0.0,
                output_peak: 0.0,
                spectrum: vec![-90.0; 96],
                generate_demo: false,
                demo_phase: 0.0,
                demo_phase2: 0.0,
            })),
            running: Arc::new(AtomicBool::new(false)),
            _input_stream: None,
            _output_stream: None,
            sample_rate: 48000,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn set_params(&self, params: EffectParams) {
        let mut s = self.shared.lock();
        s.params = params;
        s.params_dirty = true;
    }

    pub fn snapshot(&self) -> MeterSnapshot {
        let s = self.shared.lock();
        let running = self.running.load(Ordering::Relaxed);
        MeterSnapshot {
            input_peak: s.input_peak,
            output_peak: s.output_peak,
            spectrum: s.spectrum.clone(),
            running,
            listening: running && !s.generate_demo,
            sample_rate: self.sample_rate,
        }
    }

    pub fn list_input_devices() -> Vec<String> {
        let host = cpal::default_host();
        let mut names = Vec::new();
        if let Ok(devices) = host.input_devices() {
            for d in devices {
                if let Ok(name) = d.name() {
                    names.push(name);
                }
            }
        }
        if names.is_empty() {
            if let Some(d) = host.default_input_device() {
                if let Ok(name) = d.name() {
                    names.push(name);
                }
            }
        }
        names
    }

    pub fn list_output_devices() -> Vec<String> {
        let host = cpal::default_host();
        let mut names = Vec::new();
        if let Ok(devices) = host.output_devices() {
            for d in devices {
                if let Ok(name) = d.name() {
                    names.push(name);
                }
            }
        }
        if names.is_empty() {
            if let Some(d) = host.default_output_device() {
                if let Ok(name) = d.name() {
                    names.push(name);
                }
            }
        }
        names
    }

    pub fn default_input_name() -> Option<String> {
        cpal::default_host()
            .default_input_device()
            .and_then(|d| d.name().ok())
    }

    pub fn default_output_name() -> Option<String> {
        cpal::default_host()
            .default_output_device()
            .and_then(|d| d.name().ok())
    }

    fn find_input(name: Option<&str>) -> Result<Device> {
        let host = cpal::default_host();
        if let Some(want) = name {
            if let Ok(devices) = host.input_devices() {
                for d in devices {
                    if d.name().ok().as_deref() == Some(want) {
                        return Ok(d);
                    }
                }
            }
            return Err(anyhow!("Input device not found: {want}"));
        }
        host.default_input_device()
            .ok_or_else(|| anyhow!("No default input device found"))
    }

    fn find_output(name: Option<&str>) -> Result<Device> {
        let host = cpal::default_host();
        if let Some(want) = name {
            if let Ok(devices) = host.output_devices() {
                for d in devices {
                    if d.name().ok().as_deref() == Some(want) {
                        return Ok(d);
                    }
                }
            }
            return Err(anyhow!("Output device not found: {want}"));
        }
        host.default_output_device()
            .ok_or_else(|| anyhow!("No default output device found"))
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self._input_stream = None;
        self._output_stream = None;
        let mut s = self.shared.lock();
        s.generate_demo = false;
        s.input_peak = 0.0;
        s.output_peak = 0.0;
    }

    pub fn start_demo(&mut self, output_name: Option<&str>) -> Result<()> {
        self.stop();
        self.start_duplex(true, None, output_name)
    }

    pub fn start_processing(
        &mut self,
        input_name: Option<&str>,
        output_name: Option<&str>,
    ) -> Result<()> {
        self.stop();
        self.start_duplex(false, input_name, output_name)
    }

    fn start_duplex(
        &mut self,
        demo: bool,
        input_name: Option<&str>,
        output_name: Option<&str>,
    ) -> Result<()> {
        let output_device = Self::find_output(output_name)?;
        let out_config = output_device
            .default_output_config()
            .context("Could not open selected output device")?;

        let sample_rate = out_config.sample_rate().0;
        let channels = out_config.channels().max(1);
        let stream_config: StreamConfig = out_config.clone().into();

        {
            let mut s = self.shared.lock();
            s.generate_demo = demo;
            s.demo_phase = 0.0;
            s.demo_phase2 = 0.0;
            s.params_dirty = true;
        }

        let input_buf: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(8192)));

        let input_stream = if !demo {
            let input_device = Self::find_input(input_name)?;
            let in_config = input_device
                .default_input_config()
                .context("Could not open selected input device")?;
            let in_config = if in_config.sample_rate() == out_config.sample_rate() {
                in_config
            } else {
                input_device
                    .supported_input_configs()
                    .context("Could not read selected input device formats")?
                    .filter(|config| {
                        config.channels() == in_config.channels()
                            && matches!(
                                config.sample_format(),
                                SampleFormat::F32 | SampleFormat::I16
                            )
                    })
                    .find_map(|config| config.try_with_sample_rate(out_config.sample_rate()))
                    .ok_or_else(|| {
                        anyhow!(
                            "Input cannot record at {sample_rate} Hz. \
                             Choose devices with matching sample rates."
                        )
                    })?
            };
            let in_channels = in_config.channels().max(1) as usize;
            let in_stream_config: StreamConfig = in_config.clone().into();
            let input_buf_c = Arc::clone(&input_buf);
            let err_fn = |e| eprintln!("EasySound input error: {e}");

            let stream = match in_config.sample_format() {
                SampleFormat::F32 => input_device.build_input_stream(
                    &in_stream_config,
                    move |data: &[f32], _| {
                        let mut buf = input_buf_c.lock();
                        for frame in data.chunks_exact(in_channels) {
                            buf.push(frame[0]);
                            buf.push(frame.get(1).copied().unwrap_or(frame[0]));
                        }
                        if buf.len() > 48000 {
                            let drain = buf.len() - 48000;
                            buf.drain(0..drain);
                        }
                    },
                    err_fn,
                    None,
                )?,
                SampleFormat::I16 => input_device.build_input_stream(
                    &in_stream_config,
                    move |data: &[i16], _| {
                        let mut buf = input_buf_c.lock();
                        for frame in data.chunks_exact(in_channels) {
                            buf.push(frame[0] as f32 / 32768.0);
                            buf.push(frame.get(1).copied().unwrap_or(frame[0]) as f32 / 32768.0);
                        }
                        if buf.len() > 48000 {
                            let drain = buf.len() - 48000;
                            buf.drain(0..drain);
                        }
                    },
                    err_fn,
                    None,
                )?,
                other => return Err(anyhow!("Unsupported input format: {other:?}")),
            };
            stream.play()?;
            Some(stream)
        } else {
            None
        };

        let shared_o = Arc::clone(&self.shared);
        let running_o = Arc::clone(&self.running);
        let input_buf_o = Arc::clone(&input_buf);
        let out_channels = channels as usize;
        let err_fn = |e| eprintln!("EasySound output error: {e}");

        let mut chain = EffectChain::new(sample_rate as f32);
        let mut spectrum = SpectrumAnalyzer::new(2048, 96, sample_rate as f32);
        let mut left = Vec::new();
        let mut right = Vec::new();

        let mut write_out = move |out: &mut [f32]| {
            if !running_o.load(Ordering::Relaxed) {
                out.fill(0.0);
                return;
            }

            let frames = out.len() / out_channels;
            left.resize(frames, 0.0);
            right.resize(frames, 0.0);

            {
                let mut s = shared_o.lock();
                if s.params_dirty {
                    chain.set_params(s.params.clone());
                    s.params_dirty = false;
                }

                if s.generate_demo {
                    let sr = sample_rate as f32;
                    for i in 0..frames {
                        s.demo_phase += 220.0 * 2.0 * std::f32::consts::PI / sr;
                        s.demo_phase2 += 330.0 * 2.0 * std::f32::consts::PI / sr;
                        if s.demo_phase > std::f32::consts::TAU {
                            s.demo_phase -= std::f32::consts::TAU;
                        }
                        if s.demo_phase2 > std::f32::consts::TAU {
                            s.demo_phase2 -= std::f32::consts::TAU;
                        }
                        let sample = (s.demo_phase.sin() * 0.2)
                            + (s.demo_phase2.sin() * 0.12)
                            + ((s.demo_phase * 0.5).sin() * 0.08);
                        left[i] = sample;
                        right[i] = sample * 0.96;
                    }
                } else {
                    let mut buf = input_buf_o.lock();
                    let need = frames * 2;
                    if buf.len() >= need {
                        for i in 0..frames {
                            left[i] = buf[i * 2];
                            right[i] = buf[i * 2 + 1];
                        }
                        buf.drain(0..need);
                    } else {
                        left.fill(0.0);
                        right.fill(0.0);
                    }
                }

                let mut in_peak = 0.0f32;
                for i in 0..frames {
                    in_peak = in_peak.max(left[i].abs()).max(right[i].abs());
                }
                s.input_peak = s.input_peak * 0.9 + in_peak * 0.1;
            }

            chain.process(&mut left, &mut right);

            let mut out_peak = 0.0f32;
            for i in 0..frames {
                out_peak = out_peak.max(left[i].abs()).max(right[i].abs());
                let l = left[i].clamp(-1.0, 1.0);
                let r = right[i].clamp(-1.0, 1.0);
                if out_channels == 1 {
                    out[i] = (l + r) * 0.5;
                } else {
                    out[i * out_channels] = l;
                    out[i * out_channels + 1] = r;
                    for c in 2..out_channels {
                        out[i * out_channels + c] = 0.0;
                    }
                }
            }

            let mut mono = vec![0.0f32; frames];
            for i in 0..frames {
                mono[i] = (left[i] + right[i]) * 0.5;
            }
            spectrum.push_mono(&mono);

            let mut s = shared_o.lock();
            s.output_peak = s.output_peak * 0.9 + out_peak * 0.1;
            s.spectrum = spectrum.bins.clone();
        };

        let stream = match out_config.sample_format() {
            SampleFormat::F32 => output_device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _| write_out(data),
                err_fn,
                None,
            )?,
            SampleFormat::I16 => {
                let mut write_out = write_out;
                output_device.build_output_stream(
                    &stream_config,
                    move |data: &mut [i16], _| {
                        let mut tmp = vec![0.0f32; data.len()];
                        write_out(&mut tmp);
                        for (o, s) in data.iter_mut().zip(tmp.iter()) {
                            *o = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            other => return Err(anyhow!("Unsupported output format: {other:?}")),
        };
        stream.play()?;
        self._input_stream = input_stream;
        self._output_stream = Some(stream);
        self.sample_rate = sample_rate;
        self.running.store(true, Ordering::SeqCst);
        Ok(())
    }
}
