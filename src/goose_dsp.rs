use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui;
use hound;
use rfd::FileDialog;
use std::sync::{Arc, Mutex};

use crate::goose_dsp_core::process_audio;

#[derive(Default)]
pub struct GooseDsp {
    available_input_devices: Vec<String>,
    available_output_devices: Vec<String>,
    selected_input_device: Option<String>,
    selected_output_device: Option<String>,
    stream: Option<cpal::Stream>,
    output_stream: Option<cpal::Stream>,
    stream_config: Arc<Mutex<Option<cpal::StreamConfig>>>,
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
        let host = cpal::default_host();
        let input_devices = host
            .input_devices()
            .unwrap()
            .map(|d| d.name().unwrap())
            .collect();
        let output_devices = host
            .output_devices()
            .unwrap()
            .map(|d| d.name().unwrap())
            .collect();

        GooseDsp {
            selected_input_device: None,
            selected_output_device: None,
            stream: None,
            output_stream: None,
            available_input_devices: input_devices,
            available_output_devices: output_devices,
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
        let host = cpal::default_host();

        if self.selected_input_device.is_none() && self.selected_output_device.is_none() {
            self.error_message = Some("Please select at least one device".to_string());
            return;
        }

        if let Some(input_device_name) = &self.selected_input_device {
            let input_device = match host
                .input_devices()
                .unwrap()
                .find(|d| d.name().unwrap() == *input_device_name)
            {
                Some(device) => device,
                None => {
                    self.error_message = Some("Selected input device not found".to_string());
                    return;
                }
            };

            let mut input_configs = input_device
                .supported_input_configs()
                .expect("Error getting input configs");
            let mut output_configs: Vec<Vec<_>> = host
                .output_devices()
                .unwrap()
                .map(|d| {
                    d.supported_output_configs()
                        .expect("Error getting output configs")
                        .collect()
                })
                .collect();

            let supported_config = input_configs
                .find(|input_config| {
                    let input_max_sample_rate = input_config.max_sample_rate();
                    let input_min_sample_rate = input_config.min_sample_rate();

                    output_configs.iter().any(|configs| {
                        configs.iter().any(|output_config| {
                            let output_max_sample_rate = output_config.max_sample_rate();
                            let output_min_sample_rate = output_config.min_sample_rate();

                            // Check if there's an overlapping sample rate range
                            input_max_sample_rate >= output_min_sample_rate
                                && input_min_sample_rate <= output_max_sample_rate
                                && input_config.channels() == output_config.channels()
                        })
                    })
                })
                .expect("No compatible configuration found");

            let sample_rate = supported_config.min_sample_rate();
            let config = supported_config.with_sample_rate(sample_rate).config();

            *self.stream_config.lock().unwrap() = Some(config.clone());

            let stream_config = Arc::clone(&self.stream_config);
            let input_volume = self.input_volume;
            let output_volume = self.output_volume;
            let overdrive_enabled = self.overdrive_enabled;
            let threshold = self.overdrive_threshold;
            let gain = self.overdrive_gain;

            let processed_audio = Arc::new(Mutex::new(Vec::new()));
            let processed_audio_clone = Arc::clone(&processed_audio);

            let input_stream = input_device
                .build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let config = stream_config.lock().unwrap();
                        let processed = process_audio(
                            data,
                            &config,
                            input_volume,
                            output_volume,
                            overdrive_enabled,
                            threshold,
                            gain,
                        );
                        *processed_audio.lock().unwrap() = processed;
                    },
                    move |err| {
                        eprintln!("Input stream error: {}", err);
                    },
                )
                .expect("Failed to build input stream");

            let output_stream = host
                .output_devices()
                .unwrap()
                .find(|d| d.name().unwrap() == *self.selected_output_device.as_ref().unwrap())
                .expect("Output device not found")
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let processed = processed_audio_clone.lock().unwrap();
                        if !processed.is_empty() {
                            data.iter_mut().zip(processed.iter().cycle()).for_each(
                                |(out, processed)| {
                                    *out = *processed;
                                },
                            );
                        }
                    },
                    move |err| {
                        eprintln!("Output stream error: {}", err);
                    },
                )
                .expect("Failed to build output stream");

            input_stream.play().unwrap();
            output_stream.play().unwrap();

            self.stream = Some(input_stream);
            self.output_stream = Some(output_stream);

            println!("Input device supported configs:");
            for config in input_device.supported_input_configs().unwrap() {
                println!("{:?}", config);
            }

            println!("Output device supported configs:");
            for config in host.output_devices().unwrap() {
                let configs: Vec<_> = config.supported_output_configs().unwrap().collect();
                println!("{:?}", configs);
            }
        }

        if let Some(output_device_name) = &self.selected_output_device {
            let output_device = match host
                .output_devices()
                .unwrap()
                .find(|d| d.name().unwrap() == *output_device_name)
            {
                Some(device) => device,
                None => {
                    self.error_message = Some("Selected output device not found".to_string());
                    return;
                }
            };
        }
    }

    fn process_wav_file(&mut self) {
        if !self.input_file_path.is_empty() && !self.output_file_path.is_empty() {
            let mut reader = hound::WavReader::open(&self.input_file_path).unwrap();
            let spec = reader.spec();
            let mut samples: Vec<f32> = reader
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

        let plot = egui::plot::Plot::new(title)
            .view_aspect(2.0)
            .show_axes([false, true])
            .show_background(true)
            .allow_scroll(false)
            .allow_boxed_zoom(false)
            .allow_drag(false)
            .allow_zoom(false);

        let step = (samples.len() / MAX_POINTS).max(1);

        let line = egui::plot::Line::new(egui::plot::PlotPoints::from_iter(
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
        if self.selected_output_device.is_none() {
            self.error_message = Some("Please select an output device".to_string());
            return;
        }

        self.playback_stream = None;

        let host = cpal::default_host();
        let output_device = host
            .output_devices()
            .unwrap()
            .find(|d| d.name().unwrap() == *self.selected_output_device.as_ref().unwrap())
            .expect("Output device not found");

        let supported_config = output_device
            .supported_output_configs()
            .unwrap()
            .find(|config| config.channels() == 1 || config.channels() == 2)
            .expect("No supported output config found");

        let sample_rate = supported_config.min_sample_rate();
        let config = supported_config.with_sample_rate(sample_rate).config();

        let samples = Arc::new(samples);
        let samples_clone = Arc::clone(&samples);
        let mut sample_index = Arc::new(Mutex::new(0));
        let sample_index_clone = Arc::clone(&sample_index);
        let channels = config.channels as usize;

        let stream = output_device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut idx = sample_index_clone.lock().unwrap();
                    for frame in data.chunks_mut(channels) {
                        if *idx >= samples_clone.len() {
                            frame.iter_mut().for_each(|sample| *sample = 0.0);
                        } else {
                            // Copy sample to all channels
                            frame.iter_mut().for_each(|sample| {
                                *sample = samples_clone[*idx] * 0.5;
                            });
                            *idx += 1;
                        }
                    }
                },
                move |err| eprintln!("Playback error: {}", err),
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
                        ui.label("Input Device:");
                        ui.label("Output Device:");
                    });

                    ui.vertical(|ui| {
                        egui::ComboBox::from_id_source("input_device")
                            .selected_text(self.selected_input_device.clone().unwrap_or_default())
                            .width(350.0)
                            .show_ui(ui, |ui| {
                                for device in &self.available_input_devices {
                                    ui.selectable_value(
                                        &mut self.selected_input_device,
                                        Some(device.clone()),
                                        device,
                                    );
                                }
                            });

                        egui::ComboBox::from_id_source("output_device")
                            .selected_text(self.selected_output_device.clone().unwrap_or_default())
                            .width(350.0)
                            .show_ui(ui, |ui| {
                                for device in &self.available_output_devices {
                                    ui.selectable_value(
                                        &mut self.selected_output_device,
                                        Some(device.clone()),
                                        device,
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
