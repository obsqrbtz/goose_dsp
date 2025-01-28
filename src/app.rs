use cpal::traits::{DeviceTrait, HostTrait};
use eframe::egui;
use std::sync::{Arc, Mutex};

mod dsp;
mod file;
mod stream;
mod ui;
mod wav;
mod waveform;

pub struct GooseDsp {
    available_devices: Vec<String>,
    selected_device: Option<String>,
    selected_input_channel: usize,
    selected_sample_rate: u32,
    selected_bit_depth: usize,
    selected_buffer_size: u32,
    stream: Option<cpal::Stream>,
    output_stream: Option<cpal::Stream>,
    stream_config: Arc<Mutex<Option<cpal::StreamConfig>>>,
    host: cpal::Host,
    input_volume: f32,
    output_volume: f32,
    overdrive_enabled: bool,
    overdrive_threshold: f32,
    overdrive_gain: f32,
    input_file_path: String,
    output_file_path: String,
    source_samples: Vec<i32>,
    processed_samples: Vec<i32>,
    selected_waveform: usize,
    error_message: Option<String>,
    is_playing_original: bool,
    is_playing_processed: bool,
    playback_stream: Option<cpal::Stream>,
}

impl GooseDsp {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let host = match cpal::host_from_id(cpal::HostId::Asio) {
            Ok(host) => host,
            Err(_) => {
                eprintln!("Failed to initialise ASIO host, falling back to default host.");
                cpal::default_host()
            }
        };
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let host = cpal::default_host();

        let devices: Vec<String> = host
            .devices()
            .unwrap()
            .filter(|d| d.supported_input_configs().is_ok() && d.supported_output_configs().is_ok())
            .map(|d| d.name().unwrap())
            .collect();

        GooseDsp {
            host,
            available_devices: devices,
            selected_device: None,
            selected_input_channel: 0,
            selected_sample_rate: 44100,
            selected_bit_depth: 32,
            selected_buffer_size: 256,
            stream: None,
            output_stream: None,
            stream_config: Arc::new(Mutex::new(None)),
            input_volume: 1.0,
            output_volume: 0.9,
            overdrive_enabled: true,
            overdrive_threshold: 0.0001,
            overdrive_gain: 500.0,
            input_file_path: String::new(),
            output_file_path: String::new(),
            source_samples: Vec::new(),
            processed_samples: Vec::new(),
            selected_waveform: 0,
            error_message: None,
            is_playing_original: false,
            is_playing_processed: false,
            playback_stream: None,
        }
    }
}

impl eframe::App for GooseDsp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_ui(ctx, _frame);
    }
}

impl Default for GooseDsp {
    fn default() -> Self {
        Self::new()
    }
}
