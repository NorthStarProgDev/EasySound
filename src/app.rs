use crate::audio::AudioEngine;
use crate::dsp::{lin_to_db, EffectId, EffectParams, FilterMode, SpectrumAnalyzer, PIPELINE};
use crate::presets::{all_presets, Preset};
use eframe::egui::{self, Color32, RichText, Sense, Stroke, Vec2};

pub struct EasySoundApp {
    engine: AudioEngine,
    params: EffectParams,
    selected: EffectId,
    presets: Vec<Preset>,
    active_preset: String,
    status: String,
    error: Option<String>,
    spectrum: Vec<f32>,
    input_peak: f32,
    output_peak: f32,
    running: bool,
    input_devices: Vec<String>,
    output_devices: Vec<String>,
    selected_input: String,
    selected_output: String,
}

impl Default for EasySoundApp {
    fn default() -> Self {
        let presets = all_presets();
        let music = presets
            .iter()
            .find(|p| p.id == "music")
            .map(|p| p.params.clone())
            .unwrap_or_default();
        let engine = AudioEngine::new();
        engine.set_params(music.clone());

        let input_devices = AudioEngine::list_input_devices();
        let output_devices = AudioEngine::list_output_devices();
        let selected_input = AudioEngine::default_input_name()
            .or_else(|| input_devices.first().cloned())
            .unwrap_or_else(|| "(no input devices)".into());
        let selected_output = AudioEngine::default_output_name()
            .or_else(|| output_devices.first().cloned())
            .unwrap_or_else(|| "(no output devices)".into());

        Self {
            engine,
            params: music,
            selected: EffectId::Equalizer,
            presets,
            active_preset: "music".into(),
            status: "Idle — pick devices, then start Demo or Listen".into(),
            error: None,
            spectrum: vec![-90.0; 96],
            input_peak: 0.0,
            output_peak: 0.0,
            running: false,
            input_devices,
            output_devices,
            selected_input,
            selected_output,
        }
    }
}

impl EasySoundApp {
    fn apply_params(&mut self) {
        self.engine.set_params(self.params.clone());
    }

    fn apply_preset(&mut self, id: &str) {
        if let Some(p) = self.presets.iter().find(|p| p.id == id) {
            self.params = p.params.clone();
            self.active_preset = id.to_string();
            self.apply_params();
            self.error = None;
        }
    }

    fn mark_custom(&mut self) {
        self.active_preset = "custom".into();
        self.apply_params();
    }

    fn refresh_devices(&mut self) {
        self.input_devices = AudioEngine::list_input_devices();
        self.output_devices = AudioEngine::list_output_devices();
        if !self.input_devices.iter().any(|d| d == &self.selected_input) {
            self.selected_input = self
                .input_devices
                .first()
                .cloned()
                .unwrap_or_else(|| "(no input devices)".into());
        }
        if !self
            .output_devices
            .iter()
            .any(|d| d == &self.selected_output)
        {
            self.selected_output = self
                .output_devices
                .first()
                .cloned()
                .unwrap_or_else(|| "(no output devices)".into());
        }
    }

    fn input_name(&self) -> Option<&str> {
        if self.selected_input.starts_with('(') {
            None
        } else {
            Some(self.selected_input.as_str())
        }
    }

    fn output_name(&self) -> Option<&str> {
        if self.selected_output.starts_with('(') {
            None
        } else {
            Some(self.selected_output.as_str())
        }
    }
}

impl eframe::App for EasySoundApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let snap = self.engine.snapshot();
        self.spectrum = snap.spectrum;
        self.input_peak = snap.input_peak;
        self.output_peak = snap.output_peak;
        self.running = snap.running;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new(crate::APP_NAME).size(28.0).strong());
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                        let center = rect.center();
                        let gold = Color32::from_rgb(243, 187, 74);
                        let points = (0..10).map(|index| {
                            let angle = index as f32 * std::f32::consts::TAU / 10.0
                                - std::f32::consts::FRAC_PI_2;
                            let radius = if index % 2 == 0 { 7.0 } else { 3.0 };
                            center + Vec2::new(angle.cos(), angle.sin()) * radius
                        }).collect::<Vec<_>>();
                        for index in 0..points.len() {
                            ui.painter().add(egui::Shape::convex_polygon(
                                vec![center, points[index], points[(index + 1) % points.len()]],
                                gold,
                                Stroke::NONE,
                            ));
                        }
                        ui.label(RichText::new("Created by NorthStar").color(gold));
                    });
                    ui.label(
                        RichText::new("A lightweight real-time audio processor for Windows")
                            .color(Color32::from_rgb(160, 170, 180)),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    meter(ui, "OUT", self.output_peak);
                    ui.add_space(12.0);
                    meter(ui, "IN", self.input_peak);
                });
            });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Input").small().color(Color32::GRAY));
                egui::ComboBox::from_id_salt("input_device")
                    .width(260.0)
                    .selected_text(&self.selected_input)
                    .show_ui(ui, |ui| {
                        for name in self.input_devices.clone() {
                            ui.selectable_value(&mut self.selected_input, name.clone(), name);
                        }
                    });

                ui.label(RichText::new("Output").small().color(Color32::GRAY));
                egui::ComboBox::from_id_salt("output_device")
                    .width(260.0)
                    .selected_text(&self.selected_output)
                    .show_ui(ui, |ui| {
                        for name in self.output_devices.clone() {
                            ui.selectable_value(&mut self.selected_output, name.clone(), name);
                        }
                    });

                if ui
                    .button("↻ Refresh")
                    .on_hover_text("Rescan audio devices")
                    .clicked()
                {
                    self.refresh_devices();
                }
            });

            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(!self.running, egui::Button::new("▶  Demo tone"))
                    .clicked()
                {
                    let out = self.output_name().map(|s| s.to_string());
                    let out_label = self.selected_output.clone();
                    match self.engine.start_demo(out.as_deref()) {
                        Ok(()) => {
                            self.status = format!(
                                "Demo → {} @ {} Hz",
                                out_label,
                                self.engine.sample_rate()
                            );
                            self.error = None;
                        }
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
                let listening = snap.listening;
                let listen = if listening {
                    egui::Button::new("Listen: On").fill(Color32::from_rgb(23, 117, 71))
                } else {
                    egui::Button::new("Listen: Off")
                };
                let clicked = ui
                    .add_enabled(!self.running || listening, listen)
                    .on_hover_text(
                        "Hear the selected input through the enabled effects. \
                         Use headphones when monitoring a microphone.",
                    )
                    .clicked();
                if clicked && listening {
                    self.engine.stop();
                    self.status = "Stopped".into();
                } else if clicked {
                    let inp = self.input_name().map(|s| s.to_string());
                    let out = self.output_name().map(|s| s.to_string());
                    let in_label = self.selected_input.clone();
                    let out_label = self.selected_output.clone();
                    match self
                        .engine
                        .start_processing(inp.as_deref(), out.as_deref())
                    {
                        Ok(()) => {
                            self.status = format!(
                                "Listening  ·  {} → {} @ {} Hz",
                                in_label,
                                out_label,
                                self.engine.sample_rate()
                            );
                            self.error = None;
                        }
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
                if self.running && !listening && ui.button("■  Stop").clicked() {
                    self.engine.stop();
                    self.status = "Stopped".into();
                }

                ui.separator();
                ui.label(RichText::new("Preset").small().color(Color32::GRAY));
                for p in &self.presets.clone() {
                    let selected = self.active_preset == p.id;
                    if ui
                        .selectable_label(selected, p.name)
                        .on_hover_text(p.blurb)
                        .clicked()
                    {
                        self.apply_preset(p.id);
                    }
                }
                if self.active_preset == "custom" {
                    ui.colored_label(Color32::from_rgb(232, 168, 56), "Custom");
                }
                ui.separator();
                if ui.button("Save Settings").clicked() {
                    match crate::settings::save(&self.params) {
                        Ok(Some(path)) => {
                            self.status = format!("Saved settings to {}", path.display());
                            self.error = None;
                        }
                        Ok(None) => {}
                        Err(error) => self.error = Some(format!("Save Settings: {error:#}")),
                    }
                }
                if ui.button("Load Settings").clicked() {
                    match crate::settings::load() {
                        Ok(Some((path, params))) => {
                            self.params = params;
                            self.mark_custom();
                            self.status = format!("Loaded settings from {}", path.display());
                            self.error = None;
                        }
                        Ok(None) => {}
                        Err(error) => self.error = Some(format!("Load Settings: {error:#}")),
                    }
                }
            });

            if let Some(err) = &self.error {
                ui.colored_label(Color32::from_rgb(226, 109, 90), err);
            }
            ui.label(
                RichText::new(&self.status)
                    .small()
                    .color(Color32::from_rgb(140, 150, 160)),
            );

            ui.add_space(8.0);
            spectrum_widget(
                ui,
                &self.spectrum,
                self.running,
                self.engine.sample_rate() as f32,
            );

            ui.add_space(8.0);
            equalizer_faders(ui, self);

            ui.add_space(8.0);
            ui.columns(2, |cols| {
                cols[0].set_width(280.0);
                egui::ScrollArea::vertical()
                    .id_salt("pipeline")
                    .show(&mut cols[0], |ui| {
                        ui.label(
                            RichText::new("EFFECTS CHAIN")
                                .small()
                                .strong()
                                .color(Color32::from_rgb(140, 150, 160)),
                        );
                        ui.add_space(4.0);
                        for (i, id) in PIPELINE.iter().enumerate() {
                            let on = self.params.is_enabled(*id);
                            let selected = self.selected == *id;
                            let response = ui.add_sized(
                                Vec2::new(ui.available_width(), 44.0),
                                egui::Button::new("")
                                    .fill(if selected {
                                        Color32::from_rgb(40, 48, 56)
                                    } else {
                                        Color32::from_rgb(28, 34, 40)
                                    })
                                    .stroke(Stroke::new(
                                        1.0_f32,
                                        if selected {
                                            Color32::from_rgb(232, 168, 56)
                                        } else {
                                            Color32::from_rgb(50, 58, 66)
                                        },
                                    )),
                            );
                            let rect = response.rect;
                            ui.painter().text(
                                egui::pos2(rect.left() + 10.0, rect.center().y - 8.0),
                                egui::Align2::LEFT_CENTER,
                                format!("{:02}", i + 1),
                                egui::FontId::monospace(11.0),
                                Color32::GRAY,
                            );
                            ui.painter().text(
                                egui::pos2(rect.left() + 36.0, rect.center().y - 8.0),
                                egui::Align2::LEFT_CENTER,
                                id.label(),
                                egui::FontId::proportional(14.0),
                                Color32::WHITE,
                            );
                            ui.painter().text(
                                egui::pos2(rect.left() + 36.0, rect.center().y + 10.0),
                                egui::Align2::LEFT_CENTER,
                                id.description(),
                                egui::FontId::proportional(11.0),
                                Color32::from_rgb(130, 140, 150),
                            );
                            let toggle = ui.allocate_rect(
                                egui::Rect::from_min_size(
                                    egui::pos2(rect.right() - 36.0, rect.top()),
                                    Vec2::new(36.0, rect.height()),
                                ),
                                Sense::click(),
                            );
                            ui.painter().circle_filled(
                                toggle.rect.center(),
                                5.0,
                                if on {
                                    Color32::from_rgb(61, 191, 154)
                                } else {
                                    Color32::DARK_GRAY
                                },
                            );
                            if response.clicked() {
                                self.selected = *id;
                            }
                            if toggle.clicked() {
                                let next = !on;
                                self.params.set_enabled(*id, next);
                                self.mark_custom();
                            }
                        }
                    });

                egui::ScrollArea::vertical()
                    .id_salt("panel")
                    .show(&mut cols[1], |ui| {
                        effect_panel(ui, self);
                    });
            });

            ui.add_space(10.0);
            ui.separator();
            ui.label(
                RichText::new(
                    "Windows tip: install VB-Audio Cable or VoiceMeeter, set apps to Cable Input, \
                     select Cable Output in EasySound, then click Listen.",
                )
                .small()
                .color(Color32::from_rgb(120, 130, 140)),
            );
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}

fn meter(ui: &mut egui::Ui, label: &str, peak: f32) {
    let db = lin_to_db(peak);
    let value = if db <= -60.0 {
        "-∞".to_string()
    } else {
        format!("{db:.1}")
    };
    ui.group(|ui| {
        ui.set_width(132.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).small().monospace().strong().color(Color32::GRAY));
            ui.add_space(6.0);
            ui.label(RichText::new(value).small().monospace());
            ui.label(RichText::new("dB").small().monospace().color(Color32::DARK_GRAY));
        });
        let norm = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(120.0, 8.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, 3.0, Color32::from_rgb(30, 36, 42));
        let fill = egui::Rect::from_min_size(rect.min, Vec2::new(rect.width() * norm, rect.height()));
        let color = if db > -3.0 {
            Color32::from_rgb(226, 109, 90)
        } else if db > -9.0 {
            Color32::from_rgb(232, 168, 56)
        } else {
            Color32::from_rgb(61, 191, 154)
        };
        ui.painter().rect_filled(fill, 3.0, color);
    });
}

fn format_hz(hz: f32) -> String {
    if hz >= 1000.0 {
        format!("{:.1} kHz", hz / 1000.0)
    } else {
        format!("{:.0} Hz", hz)
    }
}

fn spectrum_widget(ui: &mut egui::Ui, bins: &[f32], running: bool, sample_rate: f32) {
    let height = 140.0;
    let label_h = 16.0;
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let plot = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 4.0, rect.top() + 4.0),
        egui::pos2(rect.right() - 4.0, rect.bottom() - label_h),
    );

    ui.painter()
        .rect_filled(rect, 6.0, Color32::from_rgb(12, 14, 16));

    if !running || bins.is_empty() {
        ui.painter().text(
            plot.center(),
            egui::Align2::CENTER_CENTER,
            "Spectrum idle — start Demo or Listen",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(120, 130, 140),
        );
        return;
    }

    let n = bins.len();
    let gap = 1.0_f32;
    let bar_w = ((plot.width() - gap * (n as f32 - 1.0)) / n as f32).max(1.0);
    let green = Color32::from_rgb(60, 210, 90);
    let green_dim = Color32::from_rgb(30, 120, 50);

    for (i, &db) in bins.iter().enumerate() {
        let norm = ((db + 90.0) / 75.0).clamp(0.0, 1.0);
        let h = norm * plot.height();
        let x = plot.left() + i as f32 * (bar_w + gap);
        let bar = egui::Rect::from_min_max(
            egui::pos2(x, plot.bottom() - h),
            egui::pos2(x + bar_w, plot.bottom()),
        );
        // Soft filled stem + brighter tip
        ui.painter().rect_filled(bar, 0.0, green_dim);
        let tip_h = (h * 0.18).max(2.0).min(h);
        let tip = egui::Rect::from_min_max(
            egui::pos2(bar.left(), bar.top()),
            egui::pos2(bar.right(), bar.top() + tip_h),
        );
        ui.painter().rect_filled(tip, 0.0, green);
    }

    // Frequency labels along the bottom (EasyEffects-style)
    let freqs = SpectrumAnalyzer::band_frequencies(n, sample_rate.max(44_100.0));
    let label_idx = [0, n / 5, 2 * n / 5, 3 * n / 5, 4 * n / 5, n.saturating_sub(1)];
    for &i in &label_idx {
        if i >= n {
            continue;
        }
        let x = plot.left() + i as f32 * (bar_w + gap) + bar_w * 0.5;
        ui.painter().text(
            egui::pos2(x, rect.bottom() - 2.0),
            egui::Align2::CENTER_BOTTOM,
            format_hz(freqs[i]),
            egui::FontId::monospace(10.0),
            Color32::from_rgb(140, 150, 160),
        );
    }
}

fn equalizer_faders(ui: &mut egui::Ui, app: &mut EasySoundApp) {
    let mut enabled = app.params.is_enabled(EffectId::Equalizer);
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.heading("Equalizer");
        ui.label("Gain (dB)");
        if ui.checkbox(&mut enabled, "Enabled").changed() {
            app.params.set_enabled(EffectId::Equalizer, enabled);
            changed = true;
        }
    });
    ui.add_enabled_ui(enabled, |ui| {
        ui.columns(app.params.eq_bands.len(), |columns| {
            for (column, band) in columns.iter_mut().zip(&mut app.params.eq_bands) {
                column.vertical_centered(|ui| {
                    ui.spacing_mut().slider_width = 118.0;
                    ui.label(RichText::new(format!("{:+.1}", band.gain_db)).monospace());
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut band.gain_db, -12.0..=12.0)
                                .vertical()
                                .step_by(0.1)
                                .show_value(false),
                        )
                        .on_hover_text(format!("{} gain", format_hz(band.freq)))
                        .changed();
                    let frequency = if band.freq >= 1000.0 {
                        format!("{}k", band.freq / 1000.0)
                    } else {
                        format!("{:.0} Hz", band.freq)
                    };
                    ui.label(RichText::new(frequency).small());
                });
            }
        });
    });
    if changed {
        app.mark_custom();
    }
}

fn effect_panel(ui: &mut egui::Ui, app: &mut EasySoundApp) {
    let id = app.selected;
    ui.horizontal(|ui| {
        ui.heading(id.label());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut on = app.params.is_enabled(id);
            if ui.checkbox(&mut on, "Enabled").changed() {
                app.params.set_enabled(id, on);
                app.mark_custom();
            }
        });
    });
    ui.label(RichText::new(id.description()).color(Color32::GRAY));
    ui.add_space(8.0);

    let enabled = app.params.is_enabled(id);
    ui.add_enabled_ui(enabled, |ui| match id {
        EffectId::Autogain => {
            if slider(ui, &mut app.params.ag_target, -36.0..=-6.0, "Target (dB)")
                | slider(ui, &mut app.params.ag_max_gain, 0.0..=24.0, "Max gain (dB)")
            {
                app.mark_custom();
            }
        }
        EffectId::BassEnhancer => {
            if slider(ui, &mut app.params.bass_amount, 0.0..=10.0, "Amount")
                | slider(ui, &mut app.params.bass_harmonics, 0.0..=1.0, "Harmonics")
                | slider(ui, &mut app.params.bass_blend, 0.0..=1.0, "Blend")
            {
                app.mark_custom();
            }
        }
        EffectId::BassLoudness => {
            if slider(ui, &mut app.params.bl_amount, 0.0..=12.0, "Amount (dB)")
                | slider(ui, &mut app.params.bl_freq, 40.0..=200.0, "Frequency (Hz)")
            {
                app.mark_custom();
            }
        }
        EffectId::Equalizer => {
            ui.label("Bandwidth (Q)");
            let mut changed = false;
            for band in &mut app.params.eq_bands {
                changed |= slider(ui, &mut band.q, 0.3..=4.0, &format_hz(band.freq));
            }
            if changed {
                app.mark_custom();
            }
        }
        EffectId::Filter => {
            egui::ComboBox::from_label("Mode")
                .selected_text(format!("{:?}", app.params.filter_mode))
                .show_ui(ui, |ui| {
                    for mode in [
                        FilterMode::HighPass,
                        FilterMode::LowPass,
                        FilterMode::BandPass,
                        FilterMode::Notch,
                    ] {
                        if ui
                            .selectable_value(&mut app.params.filter_mode, mode, format!("{mode:?}"))
                            .changed()
                        {
                            app.mark_custom();
                        }
                    }
                });
            if slider(ui, &mut app.params.filter_freq, 20.0..=16000.0, "Frequency (Hz)")
                | slider(ui, &mut app.params.filter_q, 0.3..=8.0, "Q")
            {
                app.mark_custom();
            }
        }
        EffectId::Compressor => {
            if slider(ui, &mut app.params.comp_threshold, -60.0..=0.0, "Threshold")
                | slider(ui, &mut app.params.comp_ratio, 1.0..=20.0, "Ratio")
                | slider(ui, &mut app.params.comp_attack, 0.001..=0.2, "Attack")
                | slider(ui, &mut app.params.comp_release, 0.05..=1.0, "Release")
                | slider(ui, &mut app.params.comp_knee, 0.0..=40.0, "Knee")
                | slider(ui, &mut app.params.comp_makeup, 0.0..=24.0, "Makeup")
            {
                app.mark_custom();
            }
        }
        EffectId::MultibandCompressor => {
            if slider(ui, &mut app.params.mbc_low_thresh, -60.0..=0.0, "Low thresh")
                | slider(ui, &mut app.params.mbc_mid_thresh, -60.0..=0.0, "Mid thresh")
                | slider(ui, &mut app.params.mbc_high_thresh, -60.0..=0.0, "High thresh")
                | slider(ui, &mut app.params.mbc_ratio, 1.0..=12.0, "Ratio")
                | slider(ui, &mut app.params.mbc_mix, 0.0..=1.0, "Mix")
            {
                app.mark_custom();
            }
        }
        EffectId::Exciter => {
            if slider(ui, &mut app.params.exc_amount, 0.0..=1.0, "Amount")
                | slider(ui, &mut app.params.exc_blend, 0.0..=1.0, "Blend")
                | slider(ui, &mut app.params.exc_harmonics, 0.0..=1.0, "Harmonics")
            {
                app.mark_custom();
            }
        }
        EffectId::Crystalizer => {
            if slider(ui, &mut app.params.cry_intensity, 0.0..=1.0, "Intensity") {
                app.mark_custom();
            }
        }
        EffectId::Deesser => {
            if slider(ui, &mut app.params.de_threshold, -60.0..=-10.0, "Threshold")
                | slider(ui, &mut app.params.de_freq, 4000.0..=10000.0, "Frequency")
                | slider(ui, &mut app.params.de_range, 1.0..=24.0, "Range")
            {
                app.mark_custom();
            }
        }
        EffectId::Delay => {
            if slider(ui, &mut app.params.delay_ms, 20.0..=800.0, "Time (ms)")
                | slider(ui, &mut app.params.delay_feedback, 0.0..=0.95, "Feedback")
                | slider(ui, &mut app.params.delay_mix, 0.0..=1.0, "Mix")
                | slider(ui, &mut app.params.delay_offset_ms, 0.0..=80.0, "Stereo offset")
            {
                app.mark_custom();
            }
        }
        EffectId::Reverb => {
            if slider(ui, &mut app.params.rev_mix, 0.0..=1.0, "Mix")
                | slider(ui, &mut app.params.rev_room, 0.05..=1.0, "Room")
                | slider(ui, &mut app.params.rev_damp, 0.0..=1.0, "Damping")
                | slider(ui, &mut app.params.rev_predelay, 0.0..=0.08, "Pre-delay")
            {
                app.mark_custom();
            }
        }
        EffectId::Pitch => {
            if slider(ui, &mut app.params.pitch_semitones, -12.0..=12.0, "Semitones") {
                app.mark_custom();
            }
        }
        EffectId::StereoTools => {
            if slider(ui, &mut app.params.st_balance, -1.0..=1.0, "Balance")
                | slider(ui, &mut app.params.st_width, 0.0..=2.0, "Width")
                | slider(ui, &mut app.params.st_bass_mono, 40.0..=300.0, "Bass mono (Hz)")
            {
                app.mark_custom();
            }
        }
        EffectId::Crossfeed => {
            if slider(ui, &mut app.params.xf_strength, 0.0..=1.0, "Strength") {
                app.mark_custom();
            }
        }
        EffectId::Loudness => {
            if slider(ui, &mut app.params.loud_amount, 0.0..=1.0, "Amount") {
                app.mark_custom();
            }
        }
        EffectId::Gate => {
            if slider(ui, &mut app.params.gate_threshold, -80.0..=-10.0, "Threshold")
                | slider(ui, &mut app.params.gate_reduction, -80.0..=-6.0, "Reduction")
                | slider(ui, &mut app.params.gate_attack, 0.001..=0.05, "Attack")
                | slider(ui, &mut app.params.gate_release, 0.02..=0.5, "Release")
            {
                app.mark_custom();
            }
        }
        EffectId::MultibandGate => {
            if slider(ui, &mut app.params.mbg_threshold, -80.0..=-10.0, "Threshold")
                | slider(ui, &mut app.params.mbg_range, 6.0..=80.0, "Range")
            {
                app.mark_custom();
            }
        }
        EffectId::Maximizer => {
            if slider(ui, &mut app.params.max_drive, 0.0..=12.0, "Drive (dB)")
                | slider(ui, &mut app.params.max_ceiling, -12.0..=0.0, "Ceiling")
            {
                app.mark_custom();
            }
        }
        EffectId::Limiter => {
            if slider(ui, &mut app.params.lim_ceiling, -12.0..=0.0, "Ceiling")
                | slider(ui, &mut app.params.lim_release, 0.02..=0.5, "Release")
            {
                app.mark_custom();
            }
        }
    });
}

fn slider(ui: &mut egui::Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>, text: &str) -> bool {
    ui.add(egui::Slider::new(value, range).text(text)).changed()
}
