mod goose_dsp;
mod goose_dsp_core;

use eframe::egui;
use goose_dsp::GooseDsp;

fn main() {
    let app = GooseDsp::new();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native("Goose DSP", options, Box::new(|_| Box::new(app))).unwrap();
}
