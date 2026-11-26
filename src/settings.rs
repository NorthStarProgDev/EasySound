use crate::dsp::EffectParams;
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const FORMAT: &str = "EasySound settings";
const VERSION: u32 = 1;
const MAX_BYTES: u64 = 64 * 1024;

#[derive(Serialize)]
struct SavedSettings<'a> {
    format: &'static str,
    version: u32,
    effects: &'a EffectParams,
}

#[derive(Deserialize)]
struct LoadedSettings {
    format: String,
    version: u32,
    effects: EffectParams,
}

pub fn save(params: &EffectParams) -> Result<Option<PathBuf>> {
    let Some(mut path) = rfd::FileDialog::new()
        .set_title("Save EasySound Settings")
        .add_filter("EasySound settings", &["json"])
        .set_file_name("easysound-settings.json")
        .save_file()
    else {
        return Ok(None);
    };
    if path.extension().is_none() {
        path.set_extension("json");
    }
    let folder = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(folder)?;
    serde_json::to_writer_pretty(&mut file, &SavedSettings {
        format: FORMAT,
        version: VERSION,
        effects: params,
    })?;
    writeln!(file)?;
    file.as_file().sync_all()?;
    file.persist(&path).context("Could not replace the settings file")?;
    Ok(Some(path))
}

pub fn load() -> Result<Option<(PathBuf, EffectParams)>> {
    let Some(path) = rfd::FileDialog::new()
        .set_title("Load EasySound Settings")
        .add_filter("EasySound settings", &["json"])
        .pick_file()
    else {
        return Ok(None);
    };
    let mut bytes = Vec::new();
    std::fs::File::open(&path)?.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= MAX_BYTES, "Settings file is too large");
    let file = parse(&bytes)?;
    Ok(Some((path, file.effects)))
}

fn parse(bytes: &[u8]) -> Result<LoadedSettings> {
    let file: LoadedSettings = serde_json::from_slice(bytes).context("Invalid settings JSON")?;
    ensure!(file.format == FORMAT && file.version == VERSION, "Unsupported EasySound settings format");
    let defaults = EffectParams::default();
    let eq = &file.effects.eq_bands;
    ensure!(eq.len() == defaults.eq_bands.len(), "Unexpected number of EQ bands");
    for (band, original) in eq.iter().zip(&defaults.eq_bands) {
        ensure!(
            band.freq == original.freq && band.kind == original.kind
                && (-12.0..=12.0).contains(&band.gain_db) && (0.3..=4.0).contains(&band.q),
            "Invalid EQ settings at {} Hz", original.freq,
        );
    }
    let effects = &file.effects;
    for (value, valid, name) in [
        (effects.ag_target, -36.0..=-6.0, "ag_target"),
        (effects.ag_max_gain, 0.0..=24.0, "ag_max_gain"),
        (effects.bass_amount, 0.0..=10.0, "bass_amount"),
        (effects.bass_harmonics, 0.0..=1.0, "bass_harmonics"),
        (effects.bass_blend, 0.0..=1.0, "bass_blend"),
        (effects.bl_amount, 0.0..=12.0, "bl_amount"),
        (effects.bl_freq, 40.0..=200.0, "bl_freq"),
        (effects.filter_freq, 20.0..=16000.0, "filter_freq"),
        (effects.filter_q, 0.3..=8.0, "filter_q"),
        (effects.comp_threshold, -60.0..=0.0, "comp_threshold"),
        (effects.comp_ratio, 1.0..=20.0, "comp_ratio"),
        (effects.comp_attack, 0.001..=0.2, "comp_attack"),
        (effects.comp_release, 0.05..=1.0, "comp_release"),
        (effects.comp_knee, 0.0..=40.0, "comp_knee"),
        (effects.comp_makeup, 0.0..=24.0, "comp_makeup"),
        (effects.mbc_low_thresh, -60.0..=0.0, "mbc_low_thresh"),
        (effects.mbc_mid_thresh, -60.0..=0.0, "mbc_mid_thresh"),
        (effects.mbc_high_thresh, -60.0..=0.0, "mbc_high_thresh"),
        (effects.mbc_ratio, 1.0..=12.0, "mbc_ratio"),
        (effects.mbc_mix, 0.0..=1.0, "mbc_mix"),
        (effects.exc_amount, 0.0..=1.0, "exc_amount"),
        (effects.exc_blend, 0.0..=1.0, "exc_blend"),
        (effects.exc_harmonics, 0.0..=1.0, "exc_harmonics"),
        (effects.cry_intensity, 0.0..=1.0, "cry_intensity"),
        (effects.de_threshold, -60.0..=-10.0, "de_threshold"),
        (effects.de_freq, 4000.0..=10000.0, "de_freq"),
        (effects.de_range, 1.0..=24.0, "de_range"),
        (effects.delay_ms, 20.0..=800.0, "delay_ms"),
        (effects.delay_feedback, 0.0..=0.95, "delay_feedback"),
        (effects.delay_mix, 0.0..=1.0, "delay_mix"),
        (effects.delay_offset_ms, 0.0..=80.0, "delay_offset_ms"),
        (effects.rev_mix, 0.0..=1.0, "rev_mix"),
        (effects.rev_room, 0.05..=1.0, "rev_room"),
        (effects.rev_damp, 0.0..=1.0, "rev_damp"),
        (effects.rev_predelay, 0.0..=0.08, "rev_predelay"),
        (effects.pitch_semitones, -12.0..=12.0, "pitch_semitones"),
        (effects.st_balance, -1.0..=1.0, "st_balance"),
        (effects.st_width, 0.0..=2.0, "st_width"),
        (effects.st_bass_mono, 40.0..=300.0, "st_bass_mono"),
        (effects.xf_strength, 0.0..=1.0, "xf_strength"),
        (effects.loud_amount, 0.0..=1.0, "loud_amount"),
        (effects.gate_threshold, -80.0..=-10.0, "gate_threshold"),
        (effects.gate_reduction, -80.0..=-6.0, "gate_reduction"),
        (effects.gate_attack, 0.001..=0.05, "gate_attack"),
        (effects.gate_release, 0.02..=0.5, "gate_release"),
        (effects.mbg_threshold, -80.0..=-10.0, "mbg_threshold"),
        (effects.mbg_range, 6.0..=80.0, "mbg_range"),
        (effects.max_drive, 0.0..=12.0, "max_drive"),
        (effects.max_ceiling, -12.0..=0.0, "max_ceiling"),
        (effects.lim_ceiling, -12.0..=0.0, "lim_ceiling"),
        (effects.lim_release, 0.02..=0.5, "lim_release"),
    ] {
        ensure!(valid.contains(&value), "Invalid value for {name}");
    }
    Ok(file)
}
