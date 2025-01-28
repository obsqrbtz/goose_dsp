use crate::app::waveform;
use crate::GooseDsp;
use eframe::egui;

impl GooseDsp {
    pub fn update_ui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                0 => waveform::Plot::draw(ui, &self.source_samples, "Source Waveform"),
                1 => waveform::Plot::draw(ui, &self.processed_samples, "Processed Waveform"),
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
