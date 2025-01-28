use eframe::egui;

pub struct Plot;

impl Plot {
    pub fn draw(ui: &mut egui::Ui, samples: &[i32], title: &str) {
        const MAX_POINTS: usize = 1000;

        let plot = egui_plot::Plot::new(title)
            .view_aspect(2.0)
            .show_axes([false, true])
            .show_background(true)
            .allow_scroll(false)
            .allow_boxed_zoom(false)
            .allow_drag(false)
            .allow_zoom(false);

        let step = (samples.len() / MAX_POINTS).max(1);

        let line = egui_plot::Line::new(egui_plot::PlotPoints::from_iter(
            samples
                .iter()
                .step_by(step)
                .enumerate()
                .map(|(i, &sample)| [i as f64 * step as f64, sample as f64 / i32::MAX as f64]),
        ))
        .color(egui::Color32::BLUE);

        plot.show(ui, |plot_ui| plot_ui.line(line));
    }
}
