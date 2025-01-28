mod app;

use app::GooseDsp;

fn main() {
    let app = GooseDsp::new();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native("Goose DSP", options, Box::new(|_| Ok(Box::new(app)))).unwrap();
}
