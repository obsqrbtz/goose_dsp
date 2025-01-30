mod app;

use app::GooseDsp;
use eframe::egui;

fn main() {
    let app = GooseDsp::new();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native(
        "Goose DSP",
        options,
        Box::new(|creation_context| {
            let style = egui::Style {
                visuals: str_to_visuals(&app.theme),
                ..egui::Style::default()
            };
            creation_context.egui_ctx.set_style(style);
            Ok(Box::new(app))
        }),
    )
    .unwrap();
}

fn str_to_visuals(theme_str: &str) -> egui::style::Visuals {
    match theme_str {
        "Dark" => eframe::egui::Visuals::dark(),
        "Light" => egui::Visuals::light(),
        _ => egui::Visuals::default(),
    }
}
