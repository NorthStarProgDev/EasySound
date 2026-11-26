pub struct SpectrumAnalyzer {
    buf: Vec<f32>,
    write: usize,
    pub bins: Vec<f32>,
    hann: Vec<f32>,
    sample_rate: f32,
}

impl SpectrumAnalyzer {
    pub fn new(fft_size: usize, bins: usize, sample_rate: f32) -> Self {
        let mut hann = vec![0.0; fft_size];
        for (i, h) in hann.iter_mut().enumerate() {
            *h = 0.5
                - 0.5
                    * (2.0 * std::f32::consts::PI * i as f32 / (fft_size as f32 - 1.0)).cos();
        }
        Self {
            buf: vec![0.0; fft_size],
            write: 0,
            bins: vec![-90.0; bins],
            hann,
            sample_rate,
        }
    }

    /// Log-spaced center frequencies for UI labels (Hz).
    pub fn band_frequencies(bands: usize, sample_rate: f32) -> Vec<f32> {
        let f_min = 20.0_f32;
        let f_max = (sample_rate * 0.45).min(20_000.0);
        (0..bands)
            .map(|b| {
                let t = b as f32 / (bands.saturating_sub(1).max(1) as f32);
                f_min * (f_max / f_min).powf(t)
            })
            .collect()
    }

    pub fn push_mono(&mut self, samples: &[f32]) {
        for &s in samples {
            self.buf[self.write] = s;
            self.write += 1;
            if self.write >= self.buf.len() {
                self.write = 0;
                self.analyze();
            }
        }
    }

    fn analyze(&mut self) {
        let n = self.buf.len();
        let mut re = vec![0.0f32; n];
        for i in 0..n {
            let idx = (self.write + i) % n;
            re[i] = self.buf[idx] * self.hann[i];
        }

        let bands = self.bins.len();
        let freqs = Self::band_frequencies(bands, self.sample_rate);
        let bin_hz = self.sample_rate / n as f32;

        for (b, &freq) in freqs.iter().enumerate() {
            let k = (freq / bin_hz).round().clamp(1.0, (n / 2 - 1) as f32) as usize;
            // Average a small neighborhood for smoother bars
            let k0 = k.saturating_sub(1).max(1);
            let k1 = (k + 1).min(n / 2 - 1);
            let mut mag_acc = 0.0_f32;
            let mut count = 0.0_f32;
            for kk in k0..=k1 {
                let mut sum_re = 0.0;
                let mut sum_im = 0.0;
                let w = -2.0 * std::f32::consts::PI * kk as f32 / n as f32;
                for i in (0..n).step_by(4) {
                    let c = (w * i as f32).cos();
                    let s = (w * i as f32).sin();
                    sum_re += re[i] * c;
                    sum_im += re[i] * s;
                }
                mag_acc += (sum_re * sum_re + sum_im * sum_im).sqrt();
                count += 1.0;
            }
            let mag = (mag_acc / count.max(1.0)) / (n as f32 / 8.0);
            let db = 20.0 * mag.max(1e-9).log10();
            // Fast attack, slower release — EasyEffects-like bar feel
            if db > self.bins[b] {
                self.bins[b] = self.bins[b] * 0.35 + db * 0.65;
            } else {
                self.bins[b] = self.bins[b] * 0.85 + db * 0.15;
            }
        }
    }
}
