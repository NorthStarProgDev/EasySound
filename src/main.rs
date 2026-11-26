#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod dsp;
mod presets;
mod settings;

use app::EasySoundApp;
use eframe::egui;

const APP_NAME: &str = "EasySound V1.2";

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title(APP_NAME),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            // Dark studio look
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals = egui::Visuals::dark();
            style.visuals.window_fill = egui::Color32::from_rgb(14, 17, 20);
            style.visuals.panel_fill = egui::Color32::from_rgb(14, 17, 20);
            style.visuals.override_text_color = Some(egui::Color32::from_rgb(230, 236, 242));
            cc.egui_ctx.set_style(style);
            Ok(Box::new(EasySoundApp::default()))
        }),
    )
}
