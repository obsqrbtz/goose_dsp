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
                        let previous_device = self.selected_device.clone();
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
                        if self.selected_device != previous_device {
                            self.set_stream();
                        }

                        let previous_channel = self.selected_input_channel;
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
                        if self.selected_input_channel != previous_channel {
                            self.set_stream();
                        }

                        let previous_rate = self.selected_sample_rate;
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
                        if self.selected_sample_rate != previous_rate {
                            self.set_stream();
                        }

                        let previous_depth = self.selected_bit_depth;
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
                        if self.selected_bit_depth != previous_depth {
                            self.set_stream();
                        }

                        let previous_buffer = self.selected_buffer_size;
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
                        if self.selected_buffer_size != previous_buffer {
                            self.set_stream();
                        }
                    });

                    ui.vertical(|ui| {
                        ui.label("Volume:");
                        ui.horizontal(|ui| {
                            ui.label("Input:");
                            if ui
                                .add(egui::Slider::new(&mut self.input_volume, 0.0..=1.0))
                                .changed()
                            {
                                if let Ok(mut params) = self.audio_params.lock() {
                                    params.input_volume = self.input_volume;
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Output:");
                            if ui
                                .add(egui::Slider::new(&mut self.output_volume, 0.0..=1.0))
                                .changed()
                            {
                                if let Ok(mut params) = self.audio_params.lock() {
                                    params.output_volume = self.output_volume;
                                }
                            }
                        });
                    });
                });
            });

            ui.separator();

            ui.heading("Effects");
            if ui
                .checkbox(&mut self.overdrive_enabled, "Overdrive")
                .changed()
            {
                if let Ok(mut params) = self.audio_params.lock() {
                    params.overdrive_enabled = self.overdrive_enabled;
                }
            }
            if self.overdrive_enabled {
                ui.horizontal(|ui| {
                    ui.label("Gain:");
                    if ui
                        .add(egui::Slider::new(&mut self.overdrive_gain, 1.00..=10.00).text("Gain"))
                        .changed()
                    {
                        if let Ok(mut params) = self.audio_params.lock() {
                            params.overdrive_gain = self.overdrive_gain;
                        }
                    }
                });
            }

            ui.separator();

            if ui.checkbox(&mut self.eq_enabled, "EQ").changed() {
                if let Ok(mut params) = self.audio_params.lock() {
                    params.eq_enabled = self.eq_enabled;
                }
            }
            if self.eq_enabled {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Low");
                        if ui
                            .add(egui::Slider::new(&mut self.eq_low, 0.0..=4.0))
                            .changed()
                        {
                            if let Ok(mut params) = self.audio_params.lock() {
                                params.eq_low = self.eq_low;
                            }
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Mid");
                        if ui
                            .add(egui::Slider::new(&mut self.eq_mid, 0.0..=4.0))
                            .changed()
                        {
                            if let Ok(mut params) = self.audio_params.lock() {
                                params.eq_mid = self.eq_mid;
                            }
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("High");
                        if ui
                            .add(egui::Slider::new(&mut self.eq_high, 0.0..=4.0))
                            .changed()
                        {
                            if let Ok(mut params) = self.audio_params.lock() {
                                params.eq_high = self.eq_high;
                            }
                        }
                    });
                });
            }

            // ui.separator();

            // ui.heading("WAV File");
            // ui.horizontal(|ui| {
            //     ui.label("Source:");
            //     ui.text_edit_singleline(&mut self.input_file_path);
            //     if ui.button("Browse").clicked() {
            //         self.pick_input_file();
            //     }
            // });
            // ui.horizontal(|ui| {
            //     ui.label("Output:");
            //     ui.text_edit_singleline(&mut self.output_file_path);
            //     if ui.button("Browse").clicked() {
            //         self.pick_output_file();
            //     }
            // });
            // if ui.button("Process File").clicked() {
            //     self.process_wav_file();
            // }
            // ui.separator();

            // ui.horizontal(|ui| {
            //     ui.radio_value(&mut self.selected_waveform, 0, "Original");
            //     ui.radio_value(&mut self.selected_waveform, 1, "Processed");
            // });

            // match self.selected_waveform {
            //     0 => waveform::Plot::draw(ui, &self.source_samples, "Source Waveform"),
            //     1 => waveform::Plot::draw(ui, &self.processed_samples, "Processed Waveform"),
            //     _ => {}
            // }

            // ui.horizontal(|ui| {
            //     if ui.button("Play").clicked() {
            //         if self.selected_waveform == 0 && !self.source_samples.is_empty() {
            //             self.is_playing_original = true;
            //             self.is_playing_processed = false;
            //             self.play_samples(self.source_samples.clone());
            //         } else if self.selected_waveform == 1 && !self.processed_samples.is_empty() {
            //             self.is_playing_original = false;
            //             self.is_playing_processed = true;
            //             self.play_samples(self.processed_samples.clone());
            //         }
            //     }
            //     if ui.button("Stop").clicked() {
            //         self.is_playing_original = false;
            //         self.is_playing_processed = false;
            //         self.playback_stream = None;
            //     }
            // });

            ui.separator();

            if ui.checkbox(&mut self.gate_enabled, "Noise Gate").changed() {
                if let Ok(mut params) = self.audio_params.lock() {
                    params.gate_enabled = self.gate_enabled;
                }
            }
            if self.gate_enabled {
                ui.horizontal(|ui| {
                    ui.label("Threshold (dB):");
                    if ui
                        .add(egui::Slider::new(&mut self.gate_threshold, -60.0..=0.0))
                        .changed()
                    {
                        if let Ok(mut params) = self.audio_params.lock() {
                            params.gate_threshold = self.gate_threshold;
                        }
                    }
                });
            }
        });
    }
}
