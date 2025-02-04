use cpal::traits::{DeviceTrait, HostTrait};
use eframe::egui;
use std::sync::{Arc, Mutex};

mod dsp;
mod settings;
mod stream;
mod ui;

use dsp::params::SharedParams;

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
    output_level: Arc<Mutex<f32>>,
    overdrive_enabled: bool,
    overdrive_gain: f32,
    error_message: Option<String>,
    audio_params: SharedParams,
    eq_enabled: bool,
    eq_low: f32,
    eq_mid: f32,
    eq_high: f32,
    gate_enabled: bool,
    gate_threshold: f32,
    cabinet_enabled: bool,
    pub theme: String,
    show_about: bool,
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

        let devices: Vec<String> = match host.devices() {
            Ok(devices) => devices
                .filter(|d| {
                    d.supported_input_configs().is_ok() && d.supported_output_configs().is_ok()
                })
                .filter_map(|d| d.name().ok())
                .collect(),
            Err(e) => {
                eprintln!("Error enumerating audio devices: {}", e);
                Vec::new()
            }
        };

        let audio_params = Arc::new(Mutex::new(dsp::params::AudioParams::new(
            1.0,  // input_volume
            1.0,  // output_volume
            true, // overdrive_enabled
            0.1,  // threshold
            3.00, // gain
        )));

        let mut goose_dsp = GooseDsp {
            host,
            available_devices: devices,
            selected_device: None,
            selected_input_channel: 0,
            selected_sample_rate: 44100,
            selected_bit_depth: 32,
            selected_buffer_size: 256,
            stream: None,
            output_stream: None,
            output_level: Arc::new(Mutex::new(0.0)),
            stream_config: Arc::new(Mutex::new(None)),
            input_volume: 0.7,
            output_volume: 0.7,
            overdrive_enabled: true,
            overdrive_gain: 3.00,
            error_message: None,
            audio_params,
            eq_enabled: false,
            eq_low: 1.0,
            eq_mid: 1.0,
            eq_high: 1.0,
            gate_enabled: false,
            gate_threshold: -40.0,
            cabinet_enabled: true,
            theme: "System".to_string(),
            show_about: false,
        };

        goose_dsp.load_settings();
        goose_dsp.set_stream();
        goose_dsp
    }
}

impl eframe::App for GooseDsp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_ui(ctx, _frame);
        ctx.request_repaint();
    }
}

impl Default for GooseDsp {
    fn default() -> Self {
        Self::new()
    }
}
