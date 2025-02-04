use crate::GooseDsp;
use eframe::egui::{self, Painter, Rect, Rgba, Stroke, ThemePreference, Visuals};
use egui_knob::{self, Knob};

impl GooseDsp {
    pub fn update_ui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        let panel_frame = egui::Frame::default()
            .fill(ctx.style().visuals.window_fill())
            .stroke(ctx.style().visuals.widgets.noninteractive.bg_stroke)
            .outer_margin(egui::Vec2::new(1.0, 1.0))
            .inner_margin(10.0);

        self.show_titlebar(ctx, panel_frame);

        if self.show_about {
            self.show_about_window(ctx);
        }

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                ui.heading("Device Settings");

                if let Some(error) = &self.error_message {
                    ui.colored_label(egui::Color32::RED, error);
                }

                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    self.device_settings_ui(ui);
                });

                ui.add_space(5.0);
                ui.heading("Volume");
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    self.volume_ui(ui);
                });

                ui.heading("Effects");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    self.effects_ui(ui);
                });
            });
    }

    fn draw_volume_meter(&self, painter: &Painter, rect: Rect, level: f32) {
        let height = rect.height() * level;
        let filled_rect = Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y - height), rect.max);

        painter.rect_filled(rect, 2.0, Rgba::from_black_alpha(0.2));
        let color = Rgba::from_rgb(0.1 + level * 0.9, 0.8 - level * 0.5, 0.2);
        painter.rect_filled(filled_rect, 2.0, color);

        painter.rect_stroke(rect, 0.0, Stroke::new(1.5, Rgba::from_black_alpha(0.5)));
    }

    pub fn show_about_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("About Goose DSP")
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("Goose DSP");
                ui.label("Version 1.0.0, 2025");
                ui.label("Guitar amp simulator");
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::image_and_text(
                                egui::include_image!("../../assets/icons/github.png"),
                                " GitHub",
                            )
                            .frame(false)
                            .sense(egui::Sense::click()),
                        )
                        .clicked()
                    {
                        ui.output_mut(|o| {
                            o.open_url = Some(egui::output::OpenUrl::same_tab(
                                "https://github.com/obsqrbtz/goose_dsp",
                            ))
                        });
                    }
                });
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::image_and_text(
                                egui::include_image!("../../assets/icons/email.png"),
                                " Email",
                            )
                            .frame(false)
                            .sense(egui::Sense::click()),
                        )
                        .clicked()
                    {
                        ui.output_mut(|o| {
                            o.open_url =
                                Some(egui::output::OpenUrl::same_tab("mailto:dan@obsqrbtz.space"))
                        });
                    }
                });
                ui.separator();

                if ui.button("Close").clicked() {
                    self.show_about = false;
                }
            });
    }

    fn device_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Audio Device:");
                ui.label("Input Channel:");
                ui.label("Sample Rate:");
                ui.label("Bit Depth:");
                ui.label("Buffer Size:");
            });

            ui.vertical(|ui| {
                self.combo_box_audio_device(ui);
                self.combo_box_input_channel(ui);
                self.combo_box_sample_rate(ui);
                self.combo_box_bit_depth(ui);
                self.combo_box_buffer_size(ui);
            });

            ui.vertical(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(20.0, 100.0), egui::Sense::hover());
                    let painter = ui.painter();
                    self.draw_volume_meter(painter, rect, self.get_output_level());
                });
            })
        });
    }

    fn show_titlebar(&mut self, ctx: &egui::Context, frame: egui::Frame) {
        let logo =
            egui::Image::new(egui::include_image!("../../assets/goose.png")).max_width(128.0);
        egui::TopBottomPanel::top("titlebar")
            .frame(frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add(logo);
                    ui.add_space(5.0);
                    ui.heading("Goose DSP");

                    let titlebar_response = ui.interact(
                        ui.max_rect(),
                        ui.id().with("titlebar"),
                        egui::Sense::click_and_drag(),
                    );

                    if titlebar_response.is_pointer_button_down_on() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.button("✖").clicked() {
                            let ctx = ctx.clone();
                            std::thread::spawn(move || {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            });
                        }
                        if ui.button("🗕").clicked() {
                            let ctx = ctx.clone();
                            std::thread::spawn(move || {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            });
                        }

                        if ui.button(" ? ").clicked() {
                            self.show_about = true;
                        }

                        self.theme_toggle(ctx, ui);
                    });
                });
            });
    }

    fn theme_toggle(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let mut theme_preference = &ui.ctx().options(|opt| opt.theme_preference);
        ui.horizontal(|ui| {
            if ui
                .selectable_value(&mut theme_preference, &ThemePreference::Light, "☀ Light")
                .clicked()
            {
                self.theme = "Light".to_string();
                ctx.set_visuals(Visuals::light());
                self.save_settings();
            }
            if ui
                .selectable_value(&mut theme_preference, &ThemePreference::Dark, "🌙 Dark")
                .clicked()
            {
                self.theme = "Dark".to_string();
                ctx.set_visuals(Visuals::dark());
                self.save_settings();
            }
        });
    }

    fn volume_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .add(
                    Knob::new(
                        &mut self.input_volume,
                        0.0,
                        1.0,
                        egui_knob::KnobStyle::Wiper,
                    )
                    .with_size(30.0)
                    .with_label("In", egui_knob::LabelPosition::Bottom),
                )
                .changed()
            {
                if let Ok(mut params) = self.audio_params.lock() {
                    params.input_volume = self.input_volume;
                }
            }

            if ui
                .add(
                    Knob::new(
                        &mut self.output_volume,
                        0.0,
                        1.0,
                        egui_knob::KnobStyle::Wiper,
                    )
                    .with_size(30.0)
                    .with_label("Out", egui_knob::LabelPosition::Bottom),
                )
                .changed()
            {
                if let Ok(mut params) = self.audio_params.lock() {
                    params.output_volume = self.output_volume;
                }
            }
        });
    }

    fn effects_ui(&mut self, ui: &mut egui::Ui) {
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
                if ui
                    .add(
                        Knob::new(
                            &mut self.overdrive_gain,
                            1.00,
                            10.00,
                            egui_knob::KnobStyle::Wiper,
                        )
                        .with_size(30.0)
                        .with_label("Gain", egui_knob::LabelPosition::Bottom),
                    )
                    .changed()
                {
                    if let Ok(mut params) = self.audio_params.lock() {
                        params.overdrive_gain = self.overdrive_gain;
                    }
                }
            });
            ui.add_space(15.0);
        }

        if ui.checkbox(&mut self.cabinet_enabled, "Cabinet").changed() {
            if let Ok(mut params) = self.audio_params.lock() {
                params.cabinet_enabled = self.cabinet_enabled;
            }
        }

        if ui.checkbox(&mut self.eq_enabled, "EQ").changed() {
            if let Ok(mut params) = self.audio_params.lock() {
                params.eq_enabled = self.eq_enabled;
            }
        }
        if self.eq_enabled {
            self.eq_settings_ui(ui);
        }

        if ui.checkbox(&mut self.gate_enabled, "Noise Gate").changed() {
            if let Ok(mut params) = self.audio_params.lock() {
                params.gate_enabled = self.gate_enabled;
            }
        }
        if self.gate_enabled {
            ui.horizontal(|ui| {
                if ui
                    .add(
                        Knob::new(
                            &mut self.gate_threshold,
                            -60.0,
                            0.0,
                            egui_knob::KnobStyle::Wiper,
                        )
                        .with_size(30.0)
                        .with_label("Threshold", egui_knob::LabelPosition::Right),
                    )
                    .changed()
                {
                    if let Ok(mut params) = self.audio_params.lock() {
                        params.gate_threshold = self.gate_threshold;
                    }
                }
                ui.add_space(30.0);
            });
        }
    }

    fn eq_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui
                    .add(
                        Knob::new(&mut self.eq_low, 0.0, 4.0, egui_knob::KnobStyle::Wiper)
                            .with_size(30.0)
                            .with_label("Low", egui_knob::LabelPosition::Bottom),
                    )
                    .changed()
                {
                    if let Ok(mut params) = self.audio_params.lock() {
                        params.eq_low = self.eq_low;
                    }
                }
            });
            ui.vertical(|ui| {
                if ui
                    .add(
                        Knob::new(&mut self.eq_mid, 0.0, 4.0, egui_knob::KnobStyle::Wiper)
                            .with_size(30.0)
                            .with_label("Mid", egui_knob::LabelPosition::Bottom),
                    )
                    .changed()
                {
                    if let Ok(mut params) = self.audio_params.lock() {
                        params.eq_mid = self.eq_mid;
                    }
                }
            });
            ui.vertical(|ui| {
                if ui
                    .add(
                        Knob::new(&mut self.eq_high, 0.0, 4.0, egui_knob::KnobStyle::Wiper)
                            .with_size(30.0)
                            .with_label("High", egui_knob::LabelPosition::Bottom),
                    )
                    .changed()
                {
                    if let Ok(mut params) = self.audio_params.lock() {
                        params.eq_high = self.eq_high;
                    }
                }
            });
        });
        ui.add_space(15.0);
    }

    fn combo_box_audio_device(&mut self, ui: &mut egui::Ui) {
        let previous_device = self.selected_device.clone();
        egui::ComboBox::from_id_salt("audio_device")
            .selected_text(self.selected_device.clone().unwrap_or_default())
            .width(350.0)
            .show_ui(ui, |ui| {
                for device in &self.available_devices {
                    ui.selectable_value(&mut self.selected_device, Some(device.clone()), device);
                }
            });
        if self.selected_device != previous_device {
            self.set_stream();
            self.save_settings();
        }
    }

    fn combo_box_input_channel(&mut self, ui: &mut egui::Ui) {
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
            self.save_settings();
        }
    }

    fn combo_box_sample_rate(&mut self, ui: &mut egui::Ui) {
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
            self.save_settings();
        }
    }

    fn combo_box_bit_depth(&mut self, ui: &mut egui::Ui) {
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
            self.save_settings();
        }
    }

    fn combo_box_buffer_size(&mut self, ui: &mut egui::Ui) {
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
            self.save_settings();
        }
    }
}
