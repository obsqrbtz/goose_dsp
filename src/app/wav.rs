use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

use crate::app::dsp;
use crate::GooseDsp;

impl GooseDsp {
    pub fn process_wav_file(&mut self) {
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

            let processed_samples = dsp::process_audio(
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

    pub fn play_samples(&mut self, samples: Vec<f32>) {
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
