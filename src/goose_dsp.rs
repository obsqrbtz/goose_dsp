use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui;
use hound;
use rfd::FileDialog;
use std::sync::{Arc, Mutex};

use crate::goose_dsp_core::process_audio;

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
    source_samples: Vec<f32>,
    processed_samples: Vec<f32>,
    selected_waveform: usize,
    error_message: Option<String>,
    is_playing_original: bool,
    is_playing_processed: bool,
    playback_stream: Option<cpal::Stream>,
}

impl GooseDsp {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let host = cpal::host_from_id(cpal::HostId::Asio).expect("failed to initialise ASIO host");
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

    fn set_stream(&mut self) {
        self.error_message = None;

        if self.selected_device.is_none() {
            self.error_message = Some("Please select a device".to_string());
            return;
        }

        let device_name = self.selected_device.as_ref().unwrap();
        let device = match self
            .host
            .devices()
            .unwrap()
            .find(|d| d.name().unwrap() == *device_name)
        {
            Some(device) => device,
            None => {
                self.error_message = Some("Selected device not found".to_string());
                return;
            }
        };

        let supported_config = device
            .supported_input_configs()
            .unwrap()
            .find(|config| {
                config.channels() == 2
                    && config.sample_format() == cpal::SampleFormat::I32
                    && config.min_sample_rate().0 == self.selected_sample_rate
                    && config.max_sample_rate().0 == self.selected_sample_rate
                    && device
                        .supported_output_configs()
                        .unwrap()
                        .any(|output_config| {
                            output_config.channels() == config.channels()
                                && output_config.sample_format() == config.sample_format()
                                && output_config.min_sample_rate().0 == self.selected_sample_rate
                                && output_config.max_sample_rate().0 == self.selected_sample_rate
                        })
            })
            .expect("no supported config");

        let config = supported_config
            .with_sample_rate(cpal::SampleRate(self.selected_sample_rate))
            .config();

        println!("Using config:");
        println!("  Channels: {}", config.channels);
        println!("  Sample Rate: {} Hz", config.sample_rate.0);
        println!("  Buffer Size: {:?}", config.buffer_size);

        let stream_config = Arc::clone(&self.stream_config);
        let input_volume = self.input_volume;
        let output_volume = self.output_volume;
        let overdrive_enabled = self.overdrive_enabled;
        let threshold = self.overdrive_threshold;
        let gain = self.overdrive_gain;
        let selected_channel = self.selected_input_channel;

        let processed_audio = Arc::new(Mutex::new(Vec::new()));
        let processed_audio_clone = Arc::clone(&processed_audio);

        let input_stream = device
            .build_input_stream(
                &config,
                move |data: &[i32], _: &cpal::InputCallbackInfo| {
                    let config = stream_config.lock().unwrap();
                    let channel_data: Vec<f32> = data
                        .chunks(2)
                        .map(|chunk| chunk[selected_channel] as f32 / i32::MAX as f32)
                        .collect();

                    let processed = process_audio(
                        &channel_data,
                        &config,
                        input_volume,
                        output_volume,
                        overdrive_enabled,
                        threshold,
                        gain,
                    );
                    *processed_audio.lock().unwrap() = processed;
                },
                move |err| eprintln!("Input error: {}", err),
                None,
            )
            .expect("failed to build input stream");

        let output_stream = device
            .build_output_stream(
                &config,
                move |data: &mut [i32], _: &cpal::OutputCallbackInfo| {
                    let processed = processed_audio_clone.lock().unwrap();
                    if !processed.is_empty() {
                        for (i, sample) in data.iter_mut().enumerate() {
                            *sample = (processed[i % processed.len()] * i32::MAX as f32) as i32;
                        }
                    }
                },
                move |err| eprintln!("Output error: {}", err),
                None,
            )
            .expect("failed to build output stream");

        input_stream.play().expect("failed to start input stream");
        output_stream.play().expect("failed to start output stream");

        self.stream = Some(input_stream);
        self.output_stream = Some(output_stream);
    }

    fn process_wav_file(&mut self) {
        if !self.input_file_path.is_empty() && !self.output_file_path.is_empty() {
            let mut reader = hound::WavReader::open(&self.input_file_path).unwrap();
            let spec = reader.spec();
            // WARN: not sure if it should not be mutable. Check later.
            let samples: Vec<f32> = reader
                .samples::<i16>()
                .map(|s| s.unwrap() as f32 / i16::MAX as f32)
                .collect();

            let config = Some(cpal::StreamConfig {
                channels: spec.channels as u16,
                sample_rate: cpal::SampleRate(spec.sample_rate),
                buffer_size: cpal::BufferSize::Default,
            });

            println!("First 10 original samples:");
            for (i, sample) in samples.iter().take(10).enumerate() {
                println!("Sample {}: {}", i, sample);
            }

            let processed_samples = process_audio(
                &samples,
                &config,
                self.input_volume,
                self.output_volume,
                self.overdrive_enabled,
                self.overdrive_threshold,
                self.overdrive_gain,
            );

            println!("\nFirst 10 processed samples:");
            for (i, sample) in processed_samples.iter().take(10).enumerate() {
                println!("Sample {}: {}", i, sample);
            }

            let mut writer = hound::WavWriter::create(&self.output_file_path, spec).unwrap();
            for sample in processed_samples.iter() {
                let scaled_sample = (*sample * i16::MAX as f32) as i16;
                writer.write_sample(scaled_sample).unwrap();
            }
            writer.finalize().unwrap();

            self.source_samples = samples;
            self.processed_samples = processed_samples;
        }
    }

    fn pick_input_file(&mut self) {
        if let Some(path) = FileDialog::new().add_filter("WAV", &["wav"]).pick_file() {
            self.input_file_path = path.display().to_string();
        }
    }

    fn pick_output_file(&mut self) {
        if let Some(path) = FileDialog::new().add_filter("WAV", &["wav"]).save_file() {
            self.output_file_path = path.display().to_string();
        }
    }

    fn plot_waveform(&self, ui: &mut egui::Ui, samples: &[f32], title: &str) {
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
                .map(|(i, &sample)| [i as f64 * step as f64, sample as f64]),
        ))
        .color(egui::Color32::BLUE);

        plot.show(ui, |plot_ui| plot_ui.line(line));
    }

    fn play_samples(&mut self, samples: Vec<f32>) {
        if self.selected_device.is_none() {
            self.error_message = Some("Please select a device".to_string());
            return;
        }

        self.playback_stream = None;

        let device = self
            .host
            .devices()
            .unwrap()
            .find(|d| d.name().unwrap() == *self.selected_device.as_ref().unwrap())
            .expect("Device not found");

        let supported_config = device
            .supported_output_configs()
            .unwrap()
            .find(|config| {
                config.channels() == 2
                    && config.min_sample_rate().0 <= self.selected_sample_rate
                    && config.max_sample_rate().0 >= self.selected_sample_rate
            })
            .map(|config| config.with_sample_rate(cpal::SampleRate(self.selected_sample_rate)))
            .expect("No supported output config found");

        let config = supported_config.config();

        let samples = Arc::new(samples);
        let samples_clone = Arc::clone(&samples);
        // WARN: not sure if it should not be mutable. Check later.
        let sample_index = Arc::new(Mutex::new(0));
        let sample_index_clone = Arc::clone(&sample_index);
        let channels = config.channels as usize;

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut idx = sample_index_clone.lock().unwrap();
                    for frame in data.chunks_mut(channels) {
                        if *idx >= samples_clone.len() {
                            frame.iter_mut().for_each(|sample| *sample = 0.0);
                        } else {
                            frame.iter_mut().for_each(|sample| {
                                *sample = samples_clone[*idx] * 0.5;
                            });
                            *idx += 1;
                        }
                    }
                },
                move |err| eprintln!("Playback error: {}", err),
                None,
            )
            .unwrap();

        stream.play().unwrap();
        self.playback_stream = Some(stream);
    }
}

impl eframe::App for GooseDsp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Device settings");

            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            }

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Audio Device:");
                        ui.label("Input Channel:");
                        ui.label("Sample Rate:");
                        ui.label("Bit Depth:");
                        ui.label("Buffer Size:");
                    });

                    ui.vertical(|ui| {
                        egui::ComboBox::from_id_salt("audio_device")
                            .selected_text(self.selected_device.clone().unwrap_or_default())
                            .width(350.0)
                            .show_ui(ui, |ui| {
                                for device in &self.available_devices {
                                    ui.selectable_value(
                                        &mut self.selected_device,
                                        Some(device.clone()),
                                        device,
                                    );
                                }
                            });

                        egui::ComboBox::from_id_salt("input_channel")
                            .selected_text(format!("Channel {}", self.selected_input_channel + 1))
                            .show_ui(ui, |ui| {
                                for i in 0..2 {
                                    ui.selectable_value(
                                        &mut self.selected_input_channel,
                                        i,
                                        format!("Channel {}", i + 1),
                                    );
                                }
                            });

                        egui::ComboBox::from_id_salt("sample_rate")
                            .selected_text(format!("{} Hz", self.selected_sample_rate))
                            .show_ui(ui, |ui| {
                                for rate in [44100, 48000, 88200, 96000] {
                                    ui.selectable_value(
                                        &mut self.selected_sample_rate,
                                        rate,
                                        format!("{} Hz", rate),
                                    );
                                }
                            });

                        egui::ComboBox::from_id_salt("bit_depth")
                            .selected_text(format!("{} bit", self.selected_bit_depth))
                            .show_ui(ui, |ui| {
                                for depth in [16, 24, 32] {
                                    ui.selectable_value(
                                        &mut self.selected_bit_depth,
                                        depth,
                                        format!("{} bit", depth),
                                    );
                                }
                            });

                        egui::ComboBox::from_id_salt("buffer_size")
                            .selected_text(format!("{} samples", self.selected_buffer_size))
                            .show_ui(ui, |ui| {
                                for size in [64, 128, 256, 512, 1024] {
                                    ui.selectable_value(
                                        &mut self.selected_buffer_size,
                                        size,
                                        format!("{} samples", size),
                                    );
                                }
                            });
                    });

                    ui.vertical(|ui| {
                        ui.label("Volume:");
                        ui.horizontal(|ui| {
                            ui.label("Input:");
                            ui.add(egui::Slider::new(&mut self.input_volume, 0.0..=1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Output:");
                            ui.add(egui::Slider::new(&mut self.output_volume, 0.0..=1.0));
                        });
                    });
                });

                if ui.button("Apply").clicked() {
                    self.set_stream();
                }
            });

            ui.separator();

            ui.heading("Effects");
            ui.checkbox(&mut self.overdrive_enabled, "Overdrive");
            if self.overdrive_enabled {
                ui.horizontal(|ui| {
                    ui.label("Gain:");
                    ui.add(egui::Slider::new(&mut self.overdrive_gain, 0.0..=1000.0).text("Gain"));
                });
            }

            ui.separator();

            ui.heading("WAV File");
            ui.horizontal(|ui| {
                ui.label("Source:");
                ui.text_edit_singleline(&mut self.input_file_path);
                if ui.button("Browse").clicked() {
                    self.pick_input_file();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Output:");
                ui.text_edit_singleline(&mut self.output_file_path);
                if ui.button("Browse").clicked() {
                    self.pick_output_file();
                }
            });
            if ui.button("Process File").clicked() {
                self.process_wav_file();
            }
            ui.separator();

            ui.horizontal(|ui| {
                ui.radio_value(&mut self.selected_waveform, 0, "Original");
                ui.radio_value(&mut self.selected_waveform, 1, "Processed");
            });

            match self.selected_waveform {
                0 => self.plot_waveform(ui, &self.source_samples, "Source Waveform"),
                1 => self.plot_waveform(ui, &self.processed_samples, "Processed Waveform"),
                _ => {}
            }

            ui.horizontal(|ui| {
                if ui.button("Play").clicked() {
                    if self.selected_waveform == 0 && !self.source_samples.is_empty() {
                        self.is_playing_original = true;
                        self.is_playing_processed = false;
                        self.play_samples(self.source_samples.clone());
                    } else if self.selected_waveform == 1 && !self.processed_samples.is_empty() {
                        self.is_playing_original = false;
                        self.is_playing_processed = true;
                        self.play_samples(self.processed_samples.clone());
                    }
                }
                if ui.button("Stop").clicked() {
                    self.is_playing_original = false;
                    self.is_playing_processed = false;
                    self.playback_stream = None;
                }
            });
        });
    }
}

impl Default for GooseDsp {
    fn default() -> Self {
        Self::new()
    }
}
