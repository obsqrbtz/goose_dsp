use crate::app::dsp;
use crate::GooseDsp;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

impl GooseDsp {
    pub fn get_output_level(&self) -> f32 {
        let level = *self.output_level.lock().unwrap();
        level
    }

    pub fn set_stream(&mut self) {
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
        let selected_channel = self.selected_input_channel;
        let audio_params = Arc::clone(&self.audio_params);

        let processed_audio = Arc::new(Mutex::new(Vec::new()));
        let processed_audio_clone = Arc::clone(&processed_audio);

        let input_stream_result = device.build_input_stream(
            &config,
            move |data: &[i32], _: &cpal::InputCallbackInfo| {
                let config = stream_config.lock().unwrap();
                let channel_data: Vec<i32> = data
                    .chunks(2)
                    .map(|chunk| chunk[selected_channel])
                    .collect();

                let processed = dsp::process_audio(&channel_data, &config, &audio_params);
                *processed_audio.lock().unwrap() = processed;
            },
            move |err| eprintln!("Input error: {}", err),
            None,
        );

        if let Err(err) = input_stream_result {
            self.error_message = Some(format!("Failed to build input stream: {}", err));
            return;
        }

        let output_level_clone = Arc::clone(&self.output_level);

        let output_stream_result = device.build_output_stream(
            &config,
            move |data: &mut [i32], _: &cpal::OutputCallbackInfo| {
                let processed = processed_audio_clone.lock().unwrap();
                if !processed.is_empty() {
                    let mut sum_squares = 0.0;
                    let mut peak = 0;

                    for (i, chunk) in data.chunks_mut(2).enumerate() {
                        let sample = processed[i % processed.len()];
                        chunk[0] = sample; // Left channel
                        chunk[1] = sample; // Right channel

                        let abs_sample = sample.abs();
                        if abs_sample > peak {
                            peak = abs_sample;
                        }

                        sum_squares += (sample as f32).powi(2);
                    }

                    let rms = (sum_squares / processed.len() as f32).sqrt();
                    let peak_level = peak as f32 / i32::MAX as f32; // Normalize to 0.0 - 1.0
                    let rms_level = rms / i32::MAX as f32; // Normalize to 0.0 - 1.0

                    *output_level_clone.lock().unwrap() = rms_level.max(peak_level);
                }
            },
            move |err| eprintln!("Output error: {}", err),
            None,
        );

        if let Err(err) = output_stream_result {
            self.error_message = Some(format!("Failed to build output stream: {}", err));
            return;
        }

        let input_stream = input_stream_result.expect("failed to build input stream");
        let output_stream = output_stream_result.expect("failed to build output stream");

        input_stream.play().expect("failed to start input stream");
        output_stream.play().expect("failed to start output stream");

        self.stream = Some(input_stream);
        self.output_stream = Some(output_stream);
    }
}
