use serde::{Deserialize, Serialize};
use std::io::Write;
use toml;

use crate::GooseDsp;

#[derive(Serialize, Deserialize)]
struct Settings {
    device: DeviceSettings,
    appearance: AppearanceSettings,
}

#[derive(Serialize, Deserialize)]
struct DeviceSettings {
    selected_device: String,
    selected_input_channel: usize,
    selected_sample_rate: u32,
    selected_bit_depth: usize,
    selected_buffer_size: u32,
}

#[derive(Serialize, Deserialize)]
struct AppearanceSettings {
    theme: String,
}

impl GooseDsp {
    pub fn load_settings(&mut self) {
        let settings_path = dirs::home_dir().unwrap().join(".goose_dsp/settings.toml");

        if let Ok(settings_str) = std::fs::read_to_string(settings_path) {
            if let Ok(settings) = toml::de::from_str::<Settings>(&settings_str) {
                self.selected_device = Some(settings.device.selected_device);
                self.selected_input_channel = settings.device.selected_input_channel;
                self.selected_sample_rate = settings.device.selected_sample_rate;
                self.selected_bit_depth = settings.device.selected_bit_depth;
                self.selected_buffer_size = settings.device.selected_buffer_size;
                self.theme = settings.appearance.theme;
            }
        } else {
            eprintln!("Could not load settings from file, using defaults.");
        }
    }

    pub fn save_settings(&self) {
        let settings_dir = dirs::home_dir().unwrap().join(".goose_dsp");
        std::fs::create_dir_all(&settings_dir).expect("Unable to create settings directory");

        let settings_path = settings_dir.join("settings.toml");

        let settings = toml::to_string(&Settings {
            device: DeviceSettings {
                selected_device: self.selected_device.clone().unwrap_or_default(),
                selected_input_channel: self.selected_input_channel,
                selected_sample_rate: self.selected_sample_rate,
                selected_bit_depth: self.selected_bit_depth,
                selected_buffer_size: self.selected_buffer_size,
            },
            appearance: AppearanceSettings {
                theme: self.theme.clone(),
            },
        })
        .expect("Failed to serialize settings");

        let mut file =
            std::fs::File::create(settings_path).expect("Unable to create settings file");
        file.write_all(settings.as_bytes())
            .expect("Unable to write to settings file");
    }
}
