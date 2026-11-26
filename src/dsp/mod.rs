mod biquad;
mod effects;
mod spectrum;

pub use effects::*;
pub use spectrum::SpectrumAnalyzer;

#[inline]
pub fn db_to_lin(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

#[inline]
pub fn lin_to_db(lin: f32) -> f32 {
    20.0 * lin.max(1e-8).log10()
}

#[inline]
pub fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    v.max(lo).min(hi)
}

#[inline]
pub fn soft_clip(x: f32) -> f32 {
    x / (1.0 + x.abs())
}
