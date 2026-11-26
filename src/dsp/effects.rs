use super::biquad::Biquad;
use super::{clamp, db_to_lin, soft_clip};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectId {
    Autogain,
    BassEnhancer,
    BassLoudness,
    Equalizer,
    Filter,
    Compressor,
    MultibandCompressor,
    Exciter,
    Crystalizer,
    Deesser,
    Delay,
    Reverb,
    Pitch,
    StereoTools,
    Crossfeed,
    Loudness,
    Gate,
    MultibandGate,
    Maximizer,
    Limiter,
}

pub const PIPELINE: &[EffectId] = &[
    EffectId::Autogain,
    EffectId::BassEnhancer,
    EffectId::BassLoudness,
    EffectId::Equalizer,
    EffectId::Filter,
    EffectId::Compressor,
    EffectId::MultibandCompressor,
    EffectId::Exciter,
    EffectId::Crystalizer,
    EffectId::Deesser,
    EffectId::Delay,
    EffectId::Reverb,
    EffectId::Pitch,
    EffectId::StereoTools,
    EffectId::Crossfeed,
    EffectId::Loudness,
    EffectId::Gate,
    EffectId::MultibandGate,
    EffectId::Maximizer,
    EffectId::Limiter,
];

impl EffectId {
    pub fn label(self) -> &'static str {
        match self {
            Self::Autogain => "Autogain",
            Self::BassEnhancer => "Bass Enhancer",
            Self::BassLoudness => "Bass Loudness",
            Self::Equalizer => "Equalizer",
            Self::Filter => "Filter",
            Self::Compressor => "Compressor",
            Self::MultibandCompressor => "Multiband Compressor",
            Self::Exciter => "Exciter",
            Self::Crystalizer => "Crystalizer",
            Self::Deesser => "Deesser",
            Self::Delay => "Delay",
            Self::Reverb => "Reverb",
            Self::Pitch => "Pitch",
            Self::StereoTools => "Stereo Tools",
            Self::Crossfeed => "Crossfeed",
            Self::Loudness => "Loudness",
            Self::Gate => "Noise Gate",
            Self::MultibandGate => "Multiband Gate",
            Self::Maximizer => "Maximizer",
            Self::Limiter => "Limiter",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Autogain => "Normalize loudness toward a target level",
            Self::BassEnhancer => "Add low-end weight and harmonics",
            Self::BassLoudness => "Psychoacoustic bass compensation",
            Self::Equalizer => "Shape tone across 16 parametric bands",
            Self::Filter => "High-pass, low-pass, band-pass or notch",
            Self::Compressor => "Control dynamics and punch",
            Self::MultibandCompressor => "Compress low / mid / high independently",
            Self::Exciter => "Add high-frequency harmonic sparkle",
            Self::Crystalizer => "Expand transient detail and clarity",
            Self::Deesser => "Tame harsh sibilance",
            Self::Delay => "Echo with feedback and stereo offset",
            Self::Reverb => "Place the signal in a room",
            Self::Pitch => "Shift pitch without changing tempo",
            Self::StereoTools => "Balance, width and mid/side control",
            Self::Crossfeed => "Headphone speaker-like imaging",
            Self::Loudness => "Equal-loudness contour compensation",
            Self::Gate => "Cut noise below a threshold",
            Self::MultibandGate => "Gate low / mid / high bands",
            Self::Maximizer => "Loudness maximize with soft ceiling",
            Self::Limiter => "Brickwall peak protection",
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EqBand {
    pub freq: f32,
    pub gain_db: f32,
    pub q: f32,
    pub kind: EqKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EqKind {
    Peak,
    LowShelf,
    HighShelf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FilterMode {
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EffectParams {
    pub enabled: [bool; 20],
    // Autogain
    pub ag_target: f32,
    pub ag_max_gain: f32,
    // Bass enhancer
    pub bass_amount: f32,
    pub bass_harmonics: f32,
    pub bass_blend: f32,
    // Bass loudness
    pub bl_amount: f32,
    pub bl_freq: f32,
    // EQ
    pub eq_bands: Vec<EqBand>,
    // Filter
    pub filter_mode: FilterMode,
    pub filter_freq: f32,
    pub filter_q: f32,
    // Compressor
    pub comp_threshold: f32,
    pub comp_ratio: f32,
    pub comp_attack: f32,
    pub comp_release: f32,
    pub comp_knee: f32,
    pub comp_makeup: f32,
    // Multiband compressor
    pub mbc_low_thresh: f32,
    pub mbc_mid_thresh: f32,
    pub mbc_high_thresh: f32,
    pub mbc_ratio: f32,
    pub mbc_mix: f32,
    // Exciter
    pub exc_amount: f32,
    pub exc_blend: f32,
    pub exc_harmonics: f32,
    // Crystalizer
    pub cry_intensity: f32,
    // Deesser
    pub de_threshold: f32,
    pub de_freq: f32,
    pub de_range: f32,
    // Delay
    pub delay_ms: f32,
    pub delay_feedback: f32,
    pub delay_mix: f32,
    pub delay_offset_ms: f32,
    // Reverb
    pub rev_mix: f32,
    pub rev_room: f32,
    pub rev_damp: f32,
    pub rev_predelay: f32,
    // Pitch
    pub pitch_semitones: f32,
    // Stereo
    pub st_balance: f32,
    pub st_width: f32,
    pub st_bass_mono: f32,
    // Crossfeed
    pub xf_strength: f32,
    // Loudness
    pub loud_amount: f32,
    // Gate
    pub gate_threshold: f32,
    pub gate_attack: f32,
    pub gate_release: f32,
    pub gate_reduction: f32,
    // Multiband gate
    pub mbg_threshold: f32,
    pub mbg_range: f32,
    // Maximizer
    pub max_ceiling: f32,
    pub max_drive: f32,
    // Limiter
    pub lim_ceiling: f32,
    pub lim_release: f32,
}

impl Default for EffectParams {
    fn default() -> Self {
        let mut enabled = [false; 20];
        enabled[EffectId::Equalizer as usize] = true;
        enabled[EffectId::Compressor as usize] = true;
        enabled[EffectId::Limiter as usize] = true;
        Self {
            enabled,
            ag_target: -18.0,
            ag_max_gain: 12.0,
            bass_amount: 4.0,
            bass_harmonics: 0.35,
            bass_blend: 0.5,
            bl_amount: 3.0,
            bl_freq: 100.0,
            eq_bands: [
                32.0, 45.0, 64.0, 90.0, 125.0, 180.0, 250.0, 350.0,
                500.0, 750.0, 1000.0, 2000.0, 3000.0, 4000.0, 6000.0, 12000.0,
            ]
            .into_iter()
            .map(|freq| {
                let kind = match freq {
                    32.0 => EqKind::LowShelf,
                    12000.0 => EqKind::HighShelf,
                    _ => EqKind::Peak,
                };
                EqBand {
                    freq,
                    gain_db: 0.0,
                    q: if kind == EqKind::Peak { 1.0 } else { 0.7 },
                    kind,
                }
            })
            .collect(),
            filter_mode: FilterMode::HighPass,
            filter_freq: 30.0,
            filter_q: 0.7,
            comp_threshold: -24.0,
            comp_ratio: 3.0,
            comp_attack: 0.012,
            comp_release: 0.25,
            comp_knee: 12.0,
            comp_makeup: 2.0,
            mbc_low_thresh: -24.0,
            mbc_mid_thresh: -22.0,
            mbc_high_thresh: -20.0,
            mbc_ratio: 2.5,
            mbc_mix: 1.0,
            exc_amount: 0.4,
            exc_blend: 0.35,
            exc_harmonics: 0.5,
            cry_intensity: 0.4,
            de_threshold: -30.0,
            de_freq: 6500.0,
            de_range: 6.0,
            delay_ms: 180.0,
            delay_feedback: 0.25,
            delay_mix: 0.2,
            delay_offset_ms: 15.0,
            rev_mix: 0.18,
            rev_room: 0.45,
            rev_damp: 0.5,
            rev_predelay: 0.02,
            pitch_semitones: 0.0,
            st_balance: 0.0,
            st_width: 1.0,
            st_bass_mono: 120.0,
            xf_strength: 0.4,
            loud_amount: 0.5,
            gate_threshold: -48.0,
            gate_attack: 0.005,
            gate_release: 0.15,
            gate_reduction: -60.0,
            mbg_threshold: -50.0,
            mbg_range: 40.0,
            max_ceiling: -1.0,
            max_drive: 2.0,
            lim_ceiling: -1.2,
            lim_release: 0.08,
        }
    }
}

impl EffectParams {
    pub fn is_enabled(&self, id: EffectId) -> bool {
        self.enabled[id as usize]
    }

    pub fn set_enabled(&mut self, id: EffectId, on: bool) {
        self.enabled[id as usize] = on;
    }
}

pub struct EffectChain {
    sample_rate: f32,
    params: EffectParams,
    // state
    ag_gain: f32,
    bass_lp: Biquad,
    bl_shelf: Biquad,
    eq: Vec<Biquad>,
    filter: Biquad,
    comp_env: f32,
    mbc_lp: Biquad,
    mbc_hp: Biquad,
    mbc_env: [f32; 3],
    exc_hp: Biquad,
    cry_hp: Biquad,
    cry_env: f32,
    de_bp: Biquad,
    de_env: f32,
    delay_l: Vec<f32>,
    delay_r: Vec<f32>,
    delay_idx: usize,
    rev_buf: Vec<f32>,
    rev_idx: usize,
    rev_pre: Vec<f32>,
    rev_pre_idx: usize,
    pitch_buf: Vec<f32>,
    pitch_idx: f32,
    bass_mono_lp_l: Biquad,
    bass_mono_lp_r: Biquad,
    xf_lp: Biquad,
    loud_low: Biquad,
    loud_high: Biquad,
    gate_env: f32,
    mbg_lp: Biquad,
    mbg_hp: Biquad,
    mbg_env: [f32; 3],
    lim_env: f32,
    tmp_l: Vec<f32>,
    tmp_r: Vec<f32>,
    band_l: [Vec<f32>; 3],
    band_r: [Vec<f32>; 3],
}

impl EffectChain {
    pub fn new(sample_rate: f32) -> Self {
        let mut chain = Self {
            sample_rate,
            params: EffectParams::default(),
            ag_gain: 1.0,
            bass_lp: Biquad::lowpass(sample_rate, 160.0, 0.7),
            bl_shelf: Biquad::lowshelf(sample_rate, 100.0, 0.7, 0.0),
            eq: vec![Biquad::identity(); 8],
            filter: Biquad::highpass(sample_rate, 30.0, 0.7),
            comp_env: 0.0,
            mbc_lp: Biquad::lowpass(sample_rate, 250.0, 0.707),
            mbc_hp: Biquad::highpass(sample_rate, 2500.0, 0.707),
            mbc_env: [0.0; 3],
            exc_hp: Biquad::highpass(sample_rate, 3500.0, 0.7),
            cry_hp: Biquad::highpass(sample_rate, 2000.0, 0.7),
            cry_env: 0.0,
            de_bp: Biquad::bandpass(sample_rate, 6500.0, 2.0),
            de_env: 0.0,
            delay_l: vec![0.0; (sample_rate as usize) * 2],
            delay_r: vec![0.0; (sample_rate as usize) * 2],
            delay_idx: 0,
            rev_buf: vec![0.0; (sample_rate as usize) * 3],
            rev_idx: 0,
            rev_pre: vec![0.0; (sample_rate as usize / 10).max(1)],
            rev_pre_idx: 0,
            pitch_buf: vec![0.0; 8192],
            pitch_idx: 0.0,
            bass_mono_lp_l: Biquad::lowpass(sample_rate, 120.0, 0.7),
            bass_mono_lp_r: Biquad::lowpass(sample_rate, 120.0, 0.7),
            xf_lp: Biquad::lowpass(sample_rate, 700.0, 0.7),
            loud_low: Biquad::lowshelf(sample_rate, 100.0, 0.7, 0.0),
            loud_high: Biquad::highshelf(sample_rate, 8000.0, 0.7, 0.0),
            gate_env: 0.0,
            mbg_lp: Biquad::lowpass(sample_rate, 250.0, 0.707),
            mbg_hp: Biquad::highpass(sample_rate, 2500.0, 0.707),
            mbg_env: [0.0; 3],
            lim_env: 0.0,
            tmp_l: Vec::new(),
            tmp_r: Vec::new(),
            band_l: [Vec::new(), Vec::new(), Vec::new()],
            band_r: [Vec::new(), Vec::new(), Vec::new()],
        };
        chain.rebuild_filters();
        chain
    }

    pub fn params(&self) -> &EffectParams {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut EffectParams {
        &mut self.params
    }

    pub fn set_params(&mut self, params: EffectParams) {
        self.params = params;
        self.rebuild_filters();
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn rebuild_filters(&mut self) {
        let sr = self.sample_rate;
        let p = &self.params;
        self.bass_lp = Biquad::lowpass(sr, 120.0 + p.bass_amount * 12.0, 0.7);
        self.bl_shelf = Biquad::lowshelf(sr, p.bl_freq, 0.7, p.bl_amount);
        self.eq.clear();
        for band in &p.eq_bands {
            let b = match band.kind {
                EqKind::Peak => Biquad::peaking(sr, band.freq, band.q, band.gain_db),
                EqKind::LowShelf => Biquad::lowshelf(sr, band.freq, band.q, band.gain_db),
                EqKind::HighShelf => Biquad::highshelf(sr, band.freq, band.q, band.gain_db),
            };
            self.eq.push(b);
        }
        self.filter = match p.filter_mode {
            FilterMode::LowPass => Biquad::lowpass(sr, p.filter_freq, p.filter_q),
            FilterMode::HighPass => Biquad::highpass(sr, p.filter_freq, p.filter_q),
            FilterMode::BandPass => Biquad::bandpass(sr, p.filter_freq, p.filter_q),
            FilterMode::Notch => Biquad::notch(sr, p.filter_freq, p.filter_q),
        };
        self.exc_hp = Biquad::highpass(sr, 2500.0 + (1.0 - p.exc_harmonics) * 2500.0, 0.7);
        self.de_bp = Biquad::bandpass(sr, p.de_freq, 2.5);
        self.bass_mono_lp_l = Biquad::lowpass(sr, p.st_bass_mono.max(40.0), 0.7);
        self.bass_mono_lp_r = Biquad::lowpass(sr, p.st_bass_mono.max(40.0), 0.7);
        self.loud_low = Biquad::lowshelf(sr, 100.0, 0.7, p.loud_amount * 6.0);
        self.loud_high = Biquad::highshelf(sr, 8000.0, 0.7, p.loud_amount * 3.0);
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        assert_eq!(left.len(), right.len());
        let n = left.len();
        self.tmp_l.resize(n, 0.0);
        self.tmp_r.resize(n, 0.0);
        for i in 0..3 {
            self.band_l[i].resize(n, 0.0);
            self.band_r[i].resize(n, 0.0);
        }

        for id in PIPELINE {
            if !self.params.is_enabled(*id) {
                continue;
            }
            match id {
                EffectId::Autogain => self.autogain(left, right),
                EffectId::BassEnhancer => self.bass_enhancer(left, right),
                EffectId::BassLoudness => self.bass_loudness(left, right),
                EffectId::Equalizer => self.equalizer(left, right),
                EffectId::Filter => self.filter_fx(left, right),
                EffectId::Compressor => self.compressor(left, right),
                EffectId::MultibandCompressor => self.multiband_compressor(left, right),
                EffectId::Exciter => self.exciter(left, right),
                EffectId::Crystalizer => self.crystalizer(left, right),
                EffectId::Deesser => self.deesser(left, right),
                EffectId::Delay => self.delay(left, right),
                EffectId::Reverb => self.reverb(left, right),
                EffectId::Pitch => self.pitch(left, right),
                EffectId::StereoTools => self.stereo_tools(left, right),
                EffectId::Crossfeed => self.crossfeed(left, right),
                EffectId::Loudness => self.loudness(left, right),
                EffectId::Gate => self.gate(left, right),
                EffectId::MultibandGate => self.multiband_gate(left, right),
                EffectId::Maximizer => self.maximizer(left, right),
                EffectId::Limiter => self.limiter(left, right),
            }
        }
    }

    fn autogain(&mut self, left: &mut [f32], right: &mut [f32]) {
        let mut peak = 1e-8f32;
        for (&l, &r) in left.iter().zip(right.iter()) {
            peak = peak.max(l.abs()).max(r.abs());
        }
        let db = 20.0 * peak.log10();
        let needed = clamp(
            self.params.ag_target - db,
            -self.params.ag_max_gain,
            self.params.ag_max_gain,
        );
        let target = db_to_lin(needed);
        self.ag_gain = self.ag_gain * 0.95 + target * 0.05;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l *= self.ag_gain;
            *r *= self.ag_gain;
        }
    }

    fn bass_enhancer(&mut self, left: &mut [f32], right: &mut [f32]) {
        let amount = self.params.bass_amount / 10.0;
        let blend = self.params.bass_blend * amount;
        let harm = self.params.bass_harmonics;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mono = (*l + *r) * 0.5;
            let low = self.bass_lp.process(mono);
            let driven = soft_clip(low * (1.0 + amount * 3.0));
            let harmonic = soft_clip(low * low * harm * 2.0);
            let add = (driven + harmonic) * blend;
            *l += add;
            *r += add;
        }
    }

    fn bass_loudness(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l = self.bl_shelf.process(*l);
            *r = self.bl_shelf.process(*r);
        }
    }

    fn equalizer(&mut self, left: &mut [f32], right: &mut [f32]) {
        for filter in &mut self.eq {
            for (l, r) in left.iter_mut().zip(right.iter_mut()) {
                *l = filter.process(*l);
                *r = filter.process(*r);
            }
        }
    }

    fn filter_fx(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l = self.filter.process(*l);
            *r = self.filter.process(*r);
        }
    }

    fn compressor(&mut self, left: &mut [f32], right: &mut [f32]) {
        let p = &self.params;
        let atk = (-1.0 / (p.comp_attack * self.sample_rate).max(1.0)).exp();
        let rel = (-1.0 / (p.comp_release * self.sample_rate).max(1.0)).exp();
        let makeup = db_to_lin(p.comp_makeup);
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let peak = l.abs().max(r.abs());
            let coeff = if peak > self.comp_env { atk } else { rel };
            self.comp_env = coeff * self.comp_env + (1.0 - coeff) * peak;
            let level_db = 20.0 * self.comp_env.max(1e-8).log10();
            let over = level_db - p.comp_threshold;
            let gr = if over <= -p.comp_knee / 2.0 {
                0.0
            } else if over >= p.comp_knee / 2.0 {
                over * (1.0 - 1.0 / p.comp_ratio)
            } else {
                let x = over + p.comp_knee / 2.0;
                (1.0 - 1.0 / p.comp_ratio) * x * x / (2.0 * p.comp_knee)
            };
            let g = db_to_lin(-gr) * makeup;
            *l *= g;
            *r *= g;
        }
    }

    fn split_bands(&mut self, left: &[f32], right: &[f32]) {
        let n = left.len();
        for i in 0..n {
            let l = left[i];
            let r = right[i];
            let low_l = self.mbc_lp.process(l);
            let low_r = self.mbc_lp.process(r);
            let high_l = self.mbc_hp.process(l);
            let high_r = self.mbc_hp.process(r);
            let mid_l = l - low_l - high_l;
            let mid_r = r - low_r - high_r;
            self.band_l[0][i] = low_l;
            self.band_r[0][i] = low_r;
            self.band_l[1][i] = mid_l;
            self.band_r[1][i] = mid_r;
            self.band_l[2][i] = high_l;
            self.band_r[2][i] = high_r;
        }
    }

    fn multiband_compressor(&mut self, left: &mut [f32], right: &mut [f32]) {
        self.split_bands(left, right);
        let thresholds = [
            self.params.mbc_low_thresh,
            self.params.mbc_mid_thresh,
            self.params.mbc_high_thresh,
        ];
        let ratio = self.params.mbc_ratio;
        let mix = self.params.mbc_mix;
        let n = left.len();
        for b in 0..3 {
            for i in 0..n {
                let peak = self.band_l[b][i].abs().max(self.band_r[b][i].abs());
                self.mbc_env[b] = self.mbc_env[b] * 0.995 + peak * 0.005;
                let db = 20.0 * self.mbc_env[b].max(1e-8).log10();
                let over = (db - thresholds[b]).max(0.0);
                let gr = over * (1.0 - 1.0 / ratio);
                let g = db_to_lin(-gr);
                self.band_l[b][i] *= g;
                self.band_r[b][i] *= g;
            }
        }
        for i in 0..n {
            let wet_l = self.band_l[0][i] + self.band_l[1][i] + self.band_l[2][i];
            let wet_r = self.band_r[0][i] + self.band_r[1][i] + self.band_r[2][i];
            left[i] = left[i] * (1.0 - mix) + wet_l * mix;
            right[i] = right[i] * (1.0 - mix) + wet_r * mix;
        }
    }

    fn exciter(&mut self, left: &mut [f32], right: &mut [f32]) {
        let amount = self.params.exc_amount;
        let blend = self.params.exc_blend * amount;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let hi_l = self.exc_hp.process(*l);
            let hi_r = self.exc_hp.process(*r);
            *l += soft_clip(hi_l * (1.0 + amount * 4.0)) * blend;
            *r += soft_clip(hi_r * (1.0 + amount * 4.0)) * blend;
        }
    }

    fn crystalizer(&mut self, left: &mut [f32], right: &mut [f32]) {
        let intensity = self.params.cry_intensity;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let hi = self.cry_hp.process((*l + *r) * 0.5).abs();
            self.cry_env = self.cry_env * 0.9 + hi * 0.1;
            let expand = 1.0 + (hi - self.cry_env).max(0.0) * intensity * 8.0;
            *l *= expand;
            *r *= expand;
        }
    }

    fn deesser(&mut self, left: &mut [f32], right: &mut [f32]) {
        let thr = self.params.de_threshold;
        let range = self.params.de_range;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let s = self.de_bp.process((*l + *r) * 0.5).abs();
            self.de_env = self.de_env * 0.92 + s * 0.08;
            let db = 20.0 * self.de_env.max(1e-8).log10();
            let gr = clamp(db - thr, 0.0, range);
            let g = db_to_lin(-gr);
            // Apply mainly to highs by blending
            let mono = (*l + *r) * 0.5;
            let diff_l = *l - mono;
            let diff_r = *r - mono;
            let m = mono * g + mono * (1.0 - g) * 0.35;
            *l = m + diff_l * g;
            *r = m + diff_r * g;
        }
    }

    fn delay(&mut self, left: &mut [f32], right: &mut [f32]) {
        let sr = self.sample_rate;
        let base = (self.params.delay_ms * 0.001 * sr) as usize;
        let offset = (self.params.delay_offset_ms * 0.001 * sr) as usize;
        let fb = self.params.delay_feedback.clamp(0.0, 0.95);
        let mix = self.params.delay_mix;
        let len = self.delay_l.len();
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let i_l = (self.delay_idx + len - base.min(len - 1)) % len;
            let i_r = (self.delay_idx + len - (base + offset).min(len - 1)) % len;
            let dl = self.delay_l[i_l];
            let dr = self.delay_r[i_r];
            self.delay_l[self.delay_idx] = *l + dl * fb;
            self.delay_r[self.delay_idx] = *r + dr * fb;
            *l = *l * (1.0 - mix) + dl * mix;
            *r = *r * (1.0 - mix) + dr * mix;
            self.delay_idx = (self.delay_idx + 1) % len;
        }
    }

    fn reverb(&mut self, left: &mut [f32], right: &mut [f32]) {
        let mix = self.params.rev_mix;
        let damp = self.params.rev_damp;
        let room = self.params.rev_room;
        let pre_len = ((self.params.rev_predelay * self.sample_rate) as usize).max(1);
        let taps = [
            (0.029 * room + 0.013) * self.sample_rate,
            (0.037 * room + 0.019) * self.sample_rate,
            (0.053 * room + 0.023) * self.sample_rate,
            (0.079 * room + 0.031) * self.sample_rate,
        ];
        let len = self.rev_buf.len();
        let pre_cap = self.rev_pre.len();
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mono = (*l + *r) * 0.5;
            self.rev_pre[self.rev_pre_idx % pre_cap] = mono;
            let delayed = self.rev_pre[(self.rev_pre_idx + pre_cap - pre_len.min(pre_cap - 1)) % pre_cap];
            self.rev_pre_idx = (self.rev_pre_idx + 1) % pre_cap;

            let mut wet = 0.0;
            for (t, &tap) in taps.iter().enumerate() {
                let idx = (self.rev_idx + len - (tap as usize).min(len - 1)) % len;
                let sample = self.rev_buf[idx];
                wet += sample * (0.3 / (t as f32 + 1.0));
            }
            wet *= 1.0 - damp * 0.5;
            self.rev_buf[self.rev_idx] = delayed + wet * (0.35 + room * 0.4);
            self.rev_idx = (self.rev_idx + 1) % len;
            *l = *l * (1.0 - mix * 0.7) + wet * mix;
            *r = *r * (1.0 - mix * 0.7) + wet * mix * 0.96;
        }
    }

    fn pitch(&mut self, left: &mut [f32], right: &mut [f32]) {
        let semis = self.params.pitch_semitones;
        if semis.abs() < 0.01 {
            return;
        }
        let ratio = 2f32.powf(semis / 12.0);
        let len = self.pitch_buf.len();
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mono = (*l + *r) * 0.5;
            let wi = self.pitch_idx as usize % len;
            self.pitch_buf[wi] = mono;
            let read = self.pitch_idx - ratio;
            let mut ri = read;
            while ri < 0.0 {
                ri += len as f32;
            }
            let i0 = ri as usize % len;
            let i1 = (i0 + 1) % len;
            let frac = ri.fract();
            let pitched = self.pitch_buf[i0] * (1.0 - frac) + self.pitch_buf[i1] * frac;
            *l = pitched;
            *r = pitched;
            self.pitch_idx = (self.pitch_idx + 1.0) % len as f32;
        }
    }

    fn stereo_tools(&mut self, left: &mut [f32], right: &mut [f32]) {
        let bal = self.params.st_balance.clamp(-1.0, 1.0);
        let width = self.params.st_width.clamp(0.0, 2.0);
        let lg = if bal > 0.0 { 1.0 - bal } else { 1.0 };
        let rg = if bal < 0.0 { 1.0 + bal } else { 1.0 };
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let low_l = self.bass_mono_lp_l.process(*l);
            let low_r = self.bass_mono_lp_r.process(*r);
            let low = (low_l + low_r) * 0.5;
            let hi_l = *l - low_l;
            let hi_r = *r - low_r;
            let mid = (hi_l + hi_r) * 0.5;
            let side = (hi_l - hi_r) * 0.5 * width;
            *l = (low + mid + side) * lg;
            *r = (low + mid - side) * rg;
        }
    }

    fn crossfeed(&mut self, left: &mut [f32], right: &mut [f32]) {
        let s = self.params.xf_strength.clamp(0.0, 1.0);
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let cross_l = self.xf_lp.process(*r);
            let cross_r = self.xf_lp.process(*l);
            let ol = *l * (1.0 - s * 0.35) + cross_l * s * 0.35;
            let or_ = *r * (1.0 - s * 0.35) + cross_r * s * 0.35;
            *l = ol;
            *r = or_;
        }
    }

    fn loudness(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l = self.loud_high.process(self.loud_low.process(*l));
            *r = self.loud_high.process(self.loud_low.process(*r));
        }
    }

    fn gate(&mut self, left: &mut [f32], right: &mut [f32]) {
        let thr = db_to_lin(self.params.gate_threshold);
        let red = db_to_lin(self.params.gate_reduction);
        let atk = (-1.0 / (self.params.gate_attack * self.sample_rate).max(1.0)).exp();
        let rel = (-1.0 / (self.params.gate_release * self.sample_rate).max(1.0)).exp();
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let peak = l.abs().max(r.abs());
            let open = peak > thr;
            let target = if open { 1.0 } else { red };
            let coeff = if open { atk } else { rel };
            self.gate_env = coeff * self.gate_env + (1.0 - coeff) * target;
            *l *= self.gate_env;
            *r *= self.gate_env;
        }
    }

    fn multiband_gate(&mut self, left: &mut [f32], right: &mut [f32]) {
        // reuse split with gate filters
        let n = left.len();
        for i in 0..n {
            let l = left[i];
            let r = right[i];
            let low_l = self.mbg_lp.process(l);
            let low_r = self.mbg_lp.process(r);
            let high_l = self.mbg_hp.process(l);
            let high_r = self.mbg_hp.process(r);
            self.band_l[0][i] = low_l;
            self.band_r[0][i] = low_r;
            self.band_l[1][i] = l - low_l - high_l;
            self.band_r[1][i] = r - low_r - high_r;
            self.band_l[2][i] = high_l;
            self.band_r[2][i] = high_r;
        }
        let thr = db_to_lin(self.params.mbg_threshold);
        let red = db_to_lin(-self.params.mbg_range);
        for b in 0..3 {
            for i in 0..n {
                let peak = self.band_l[b][i].abs().max(self.band_r[b][i].abs());
                self.mbg_env[b] = self.mbg_env[b] * 0.98 + peak * 0.02;
                let g = if self.mbg_env[b] > thr { 1.0 } else { red };
                self.band_l[b][i] *= g;
                self.band_r[b][i] *= g;
            }
        }
        for i in 0..n {
            left[i] = self.band_l[0][i] + self.band_l[1][i] + self.band_l[2][i];
            right[i] = self.band_r[0][i] + self.band_r[1][i] + self.band_r[2][i];
        }
    }

    fn maximizer(&mut self, left: &mut [f32], right: &mut [f32]) {
        let drive = db_to_lin(self.params.max_drive);
        let ceiling = db_to_lin(self.params.max_ceiling);
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l = soft_clip(*l * drive) * ceiling;
            *r = soft_clip(*r * drive) * ceiling;
        }
    }

    fn limiter(&mut self, left: &mut [f32], right: &mut [f32]) {
        let ceiling = db_to_lin(self.params.lim_ceiling);
        let rel = (-1.0 / (self.params.lim_release * self.sample_rate).max(1.0)).exp();
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let peak = l.abs().max(r.abs()).max(1e-8);
            let needed = (ceiling / peak).min(1.0);
            if needed < self.lim_env {
                self.lim_env = needed;
            } else {
                self.lim_env = rel * self.lim_env + (1.0 - rel) * needed;
            }
            *l *= self.lim_env;
            *r *= self.lim_env;
        }
    }
}
