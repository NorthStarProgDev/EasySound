#[derive(Clone, Copy, Debug)]
pub struct Biquad {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
    z1: f32,
    z2: f32,
}

impl Default for Biquad {
    fn default() -> Self {
        Self::identity()
    }
}

impl Biquad {
    pub fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    pub fn process_buffer(&mut self, data: &mut [f32]) {
        for s in data.iter_mut() {
            *s = self.process(*s);
        }
    }

    pub fn peaking(sr: f32, freq: f32, q: f32, gain_db: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * (freq / sr).min(0.49);
        let alpha = w0.sin() / (2.0 * q.max(0.05));
        let cos = w0.cos();
        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos;
        let a2 = 1.0 - alpha / a;
        Self::normalize(b0, b1, b2, a0, a1, a2)
    }

    pub fn lowshelf(sr: f32, freq: f32, q: f32, gain_db: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * (freq / sr).min(0.49);
        let cos = w0.cos();
        let sin = w0.sin();
        let alpha = sin / (2.0 * q.max(0.05));
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
        let b0 = a * ((a + 1.0) - (a - 1.0) * cos + two_sqrt_a_alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos - two_sqrt_a_alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos + two_sqrt_a_alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos);
        let a2 = (a + 1.0) + (a - 1.0) * cos - two_sqrt_a_alpha;
        Self::normalize(b0, b1, b2, a0, a1, a2)
    }

    pub fn highshelf(sr: f32, freq: f32, q: f32, gain_db: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * (freq / sr).min(0.49);
        let cos = w0.cos();
        let sin = w0.sin();
        let alpha = sin / (2.0 * q.max(0.05));
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
        let b0 = a * ((a + 1.0) + (a - 1.0) * cos + two_sqrt_a_alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos - two_sqrt_a_alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos + two_sqrt_a_alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos);
        let a2 = (a + 1.0) - (a - 1.0) * cos - two_sqrt_a_alpha;
        Self::normalize(b0, b1, b2, a0, a1, a2)
    }

    pub fn lowpass(sr: f32, freq: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * (freq / sr).min(0.49);
        let cos = w0.cos();
        let sin = w0.sin();
        let alpha = sin / (2.0 * q.max(0.05));
        let b0 = (1.0 - cos) / 2.0;
        let b1 = 1.0 - cos;
        let b2 = (1.0 - cos) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos;
        let a2 = 1.0 - alpha;
        Self::normalize(b0, b1, b2, a0, a1, a2)
    }

    pub fn highpass(sr: f32, freq: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * (freq / sr).min(0.49);
        let cos = w0.cos();
        let sin = w0.sin();
        let alpha = sin / (2.0 * q.max(0.05));
        let b0 = (1.0 + cos) / 2.0;
        let b1 = -(1.0 + cos);
        let b2 = (1.0 + cos) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos;
        let a2 = 1.0 - alpha;
        Self::normalize(b0, b1, b2, a0, a1, a2)
    }

    pub fn bandpass(sr: f32, freq: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * (freq / sr).min(0.49);
        let cos = w0.cos();
        let sin = w0.sin();
        let alpha = sin / (2.0 * q.max(0.05));
        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos;
        let a2 = 1.0 - alpha;
        Self::normalize(b0, b1, b2, a0, a1, a2)
    }

    pub fn notch(sr: f32, freq: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * (freq / sr).min(0.49);
        let cos = w0.cos();
        let sin = w0.sin();
        let alpha = sin / (2.0 * q.max(0.05));
        let b0 = 1.0;
        let b1 = -2.0 * cos;
        let b2 = 1.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos;
        let a2 = 1.0 - alpha;
        Self::normalize(b0, b1, b2, a0, a1, a2)
    }

    fn normalize(b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> Self {
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }
}
