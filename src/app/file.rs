use crate::GooseDsp;
use rfd::FileDialog;

impl GooseDsp {
    pub fn pick_input_file(&mut self) {
        if let Some(path) = FileDialog::new().add_filter("WAV", &["wav"]).pick_file() {
            self.input_file_path = path.display().to_string();
        }
    }

    pub fn pick_output_file(&mut self) {
        if let Some(path) = FileDialog::new().add_filter("WAV", &["wav"]).save_file() {
            self.output_file_path = path.display().to_string();
        }
    }
}
