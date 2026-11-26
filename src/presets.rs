use crate::dsp::{EffectId, EffectParams, PIPELINE};

#[derive(Clone)]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
    pub params: EffectParams,
}

fn base() -> EffectParams {
    EffectParams::default()
}

fn enable(p: &mut EffectParams, ids: &[EffectId]) {
    for id in PIPELINE {
        p.set_enabled(*id, false);
    }
    for id in ids {
        p.set_enabled(*id, true);
    }
}

fn set_eq_gains(params: &mut EffectParams, gains: [f32; 8]) {
    let original_bands = [0, 2, 4, 6, 8, 11, 14, 15];
    for (index, gain) in original_bands.into_iter().zip(gains) {
        params.eq_bands[index].gain_db = gain;
    }
}

pub fn all_presets() -> Vec<Preset> {
    let mut flat = base();
    enable(&mut flat, &[EffectId::Equalizer, EffectId::Limiter]);

    let mut music = base();
    enable(
        &mut music,
        &[
            EffectId::BassEnhancer,
            EffectId::Equalizer,
            EffectId::Compressor,
            EffectId::Limiter,
        ],
    );
    set_eq_gains(&mut music, [2.0, 3.5, 1.5, 0.5, 0.0, 0.5, 2.0, 3.0]);
    music.bass_amount = 3.0;

    let mut movies = base();
    enable(
        &mut movies,
        &[
            EffectId::BassLoudness,
            EffectId::Equalizer,
            EffectId::Compressor,
            EffectId::Loudness,
            EffectId::Limiter,
        ],
    );
    set_eq_gains(&mut movies, [3.0, 2.0, 0.0, -1.0, 1.0, 2.5, 1.0, 0.5]);

    let mut voice = base();
    enable(
        &mut voice,
        &[
            EffectId::Autogain,
            EffectId::Equalizer,
            EffectId::Compressor,
            EffectId::Deesser,
            EffectId::Gate,
            EffectId::Limiter,
        ],
    );
    set_eq_gains(&mut voice, [-4.0, -2.0, 1.0, 2.0, 3.0, 4.0, 1.0, -1.0]);

    let mut bass = base();
    enable(
        &mut bass,
        &[
            EffectId::BassEnhancer,
            EffectId::BassLoudness,
            EffectId::Equalizer,
            EffectId::Compressor,
            EffectId::Maximizer,
            EffectId::Limiter,
        ],
    );
    set_eq_gains(&mut bass, [6.0, 7.0, 3.0, 0.0, -1.0, 0.0, 1.0, 2.0]);
    bass.bass_amount = 7.0;

    let mut bright = base();
    enable(
        &mut bright,
        &[
            EffectId::Equalizer,
            EffectId::Exciter,
            EffectId::Crystalizer,
            EffectId::Compressor,
            EffectId::Limiter,
        ],
    );
    set_eq_gains(&mut bright, [-1.0, 0.0, 0.0, 0.5, 1.0, 2.0, 4.0, 5.0]);

    let mut spatial = base();
    enable(
        &mut spatial,
        &[
            EffectId::Equalizer,
            EffectId::Reverb,
            EffectId::Delay,
            EffectId::StereoTools,
            EffectId::Crossfeed,
            EffectId::Limiter,
        ],
    );
    spatial.rev_mix = 0.22;
    spatial.delay_mix = 0.15;
    spatial.st_width = 1.35;

    vec![
        Preset {
            id: "flat",
            name: "Flat",
            blurb: "Clean pass-through with limiting",
            params: flat,
        },
        Preset {
            id: "music",
            name: "Music",
            blurb: "Warm bass and gentle glue",
            params: music,
        },
        Preset {
            id: "movies",
            name: "Movies",
            blurb: "Dialogue clarity and cinematic low end",
            params: movies,
        },
        Preset {
            id: "voice",
            name: "Voice",
            blurb: "Calls and podcasts",
            params: voice,
        },
        Preset {
            id: "bass",
            name: "Bass Boost",
            blurb: "Heavy low end",
            params: bass,
        },
        Preset {
            id: "bright",
            name: "Bright",
            blurb: "Airy top end",
            params: bright,
        },
        Preset {
            id: "spatial",
            name: "Spatial",
            blurb: "Width, reverb and crossfeed",
            params: spatial,
        },
    ]
}
